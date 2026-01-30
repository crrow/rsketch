# Token Refresh 机制设计

## 概述

为 `ApiClientInner` 实现一个后台 token refresh 机制，用于在 OAuth token 过期时自动刷新，避免请求失败。

## 架构决策

### 1. Worker 生命周期
- **长期运行的后台任务**，在 `ApiClientInner::open()` 时启动（仅 OAuth）
- 生命周期与 `ApiClientInner` 绑定，通过 RAII 和 `CancellationToken` 实现优雅关闭
- Browser/NoAuth token 不启动 worker（优化）

### 2. 并发协调机制
- 使用 **CAS (Compare-And-Swap)** 确保只有一个请求触发 refresh
- 两个 `Notify` 通道：
  - `worker_notify`: 请求通知 worker 需要 refresh
  - `notify`: worker 通知等待的请求 refresh 完成
- 失败的 CAS 请求等待 `notify`，避免重复 refresh

### 3. 失败处理策略
- **Fail-fast permanently**: refresh 失败后进入 `RefreshFailed` 终态
- 所有后续请求立即失败，不重试
- 用户必须重新创建 `ApiClientInner` 并重新认证

## 状态机

```
NoNeed ──(token expired)──> NeedRefreshing ──(worker starts)──> InRefreshing
                                                                      │
                                                    ┌─────────────────┴─────────────┐
                                                    │                               │
                                              (success)                        (failure)
                                                    │                               │
                                                    ▼                               ▼
                                                NoNeed                       RefreshFailed
                                                    │                               │
                                          (requests succeed)                (all fail forever)
```

**状态转换规则:**
- 只有请求可以通过 CAS 转换 `NoNeed` → `NeedRefreshing`
- 只有 worker 可以转换 `NeedRefreshing` → `InRefreshing`
- 只有 worker 可以转换 `InRefreshing` → `NoNeed` 或 `RefreshFailed`
- `RefreshFailed` 是终态，不可恢复

## 数据结构

### TokenState Enum
```rust
#[derive(IntoPrimitive, TryFromPrimitive, Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
enum TokenState {
    NoNeed = 0,
    NeedRefreshing = 1,
    InRefreshing = 2,
    RefreshFailed = 3,
}
```

### ApiClientInner
```rust
pub struct ApiClientInner {
    generic_client: RwLock<GenericalYtmusic>,
    token_state:    Arc<AtomicU8>,              // 共享状态机
    notify:         Arc<Notify>,                 // Worker → Requests
    worker_notify:  Option<Arc<Notify>>,         // Requests → Worker (OAuth only)
    cancel_token:   CancellationToken,           // 优雅关闭
}
```

### RefreshWorker
```rust
struct RefreshWorker {
    generic_client: Arc<RwLock<GenericalYtmusic>>,
    token_state:    Arc<AtomicU8>,
    worker_notify:  Arc<Notify>,
    notify:         Arc<Notify>,
    cancel_token:   CancellationToken,
}

impl RefreshWorker {
    fn new(...) -> Self;
    fn spawn(self) -> JoinHandle<()>;
    async fn run(self);
    async fn perform_refresh(&self) -> Result<()>;
}
```

## 核心流程

### ApiClientInner::open()
1. 加载 API key
2. 创建 `GenericalYtmusic` 客户端
3. **条件性启动 worker**:
   - 如果是 `ApiKey::OAuthToken`: 创建 `worker_notify`，启动 `RefreshWorker`
   - 如果是 `Browser`/`NoAuth`: `worker_notify = None`，不启动 worker
4. 返回 `ApiClientInner`

### RefreshWorker::run() 主循环
```rust
loop {
    select! {
        _ = cancel_token.cancelled() => break,  // 优雅关闭
        _ = worker_notify.notified() => {
            let state = token_state.load(Ordering::Acquire);
            if state == TokenState::NeedRefreshing {
                // 转换到 InRefreshing
                token_state.store(TokenState::InRefreshing as u8, Ordering::Release);

                // 执行 refresh
                let mut client = generic_client.write();
                match client.refresh_token().await {
                    Ok(Some(_)) => {
                        token_state.store(TokenState::NoNeed as u8, Ordering::Release);
                    }
                    Ok(None) => {
                        // Browser/NoAuth - 不应该发生
                        token_state.store(TokenState::NoNeed as u8, Ordering::Release);
                    }
                    Err(_) => {
                        // 失败 - 终态
                        token_state.store(TokenState::RefreshFailed as u8, Ordering::Release);
                    }
                }

                // 唤醒所有等待的请求
                notify.notify_waiters();
            }
        }
    }
}
```

### query_api_with_retry() 请求流程
```rust
loop {
    // 1. 检查状态
    let state = token_state.load(Ordering::Acquire);
    match state {
        TokenState::RefreshFailed => return Err(Error::TokenRefreshFailed),
        TokenState::InRefreshing | TokenState::NeedRefreshing => {
            self.notify.notified().await;
            continue;
        }
        TokenState::NoNeed => { /* 继续 */ }
    }

    // 2. 执行请求
    let result = self.generic_client.read()
        .query_browser_or_oauth(query.borrow())
        .await;

    // 3. 处理结果
    match result {
        Ok(output) => return Ok(output),
        Err(e) => {
            if let Some(ErrorKind::OAuthTokenExpired { .. }) = e.kind() {
                // 先注册通知（避免 lost wakeup）
                let notified = self.notify.notified();

                // 尝试 CAS
                let cas_result = self.token_state.compare_exchange(
                    TokenState::NoNeed as u8,
                    TokenState::NeedRefreshing as u8,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                );

                if cas_result.is_ok() {
                    // 成功 - 通知 worker
                    if let Some(worker_notify) = &self.worker_notify {
                        worker_notify.notify_one();
                    }
                }

                // 等待 refresh 完成
                notified.await;
                continue;
            } else {
                return Err(e.into());
            }
        }
    }
}
```

## 关键设计点

### 1. Lost Wakeup 防护
**问题**: 如果在 `notified().await` 之前 worker 已经发出通知，请求会永久等待。

**解决**: 先调用 `notified()` 创建 future（注册等待），再执行 CAS，最后 await：
```rust
let notified = self.notify.notified();  // 注册
let cas_result = self.token_state.compare_exchange(...);
if cas_result.is_ok() { /* ... */ }
notified.await;  // 等待（不会错过通知）
```

### 2. Browser/NoAuth 优化
- 这些 token 类型不需要 refresh
- `worker_notify` 为 `None`，不启动 worker
- 节省资源，代码更清晰

### 3. 优雅关闭
- `ApiClientInner` drop 时，`CancellationToken` 自动 cancel
- Worker 在下一次 `select!` 时检测到 cancel，退出循环
- 不需要显式 join（fire-and-forget spawn）

### 4. Fail-Fast 语义
- Refresh 失败进入 `RefreshFailed` 终态
- 避免重复尝试无效的 refresh token
- 强制用户处理认证失败（重新认证）

## 错误处理

### 新增错误类型
```rust
#[derive(Snafu, Debug)]
pub enum Error {
    // ...
    #[snafu(display(
        "Token refresh failed, client is no longer usable. \
         Please recreate the client and re-authenticate."
    ))]
    TokenRefreshFailed {
        #[snafu(implicit)]
        loc: snafu::Location,
    },
}
```

### 边界情况
1. **并发 drop**: cancel_token 会优雅地停止 worker
2. **Browser/NoAuth token**: 不会遇到 `OAuthTokenExpired`，worker 不启动
3. **Token hash 验证**: 可选的后续优化，验证过期的 token 是否是当前 token

## 实现清单

- [ ] 定义 `RefreshWorker` struct
- [ ] 实现 `RefreshWorker::new()`, `spawn()`, `run()`
- [ ] 更新 `ApiClientInner` 结构，添加 `worker_notify: Option<Arc<Notify>>`
- [ ] 修改 `ApiClientInner::open()`，条件性启动 worker
- [ ] 实现 `query_api_with_retry()` 完整流程
- [ ] 添加 `Error::TokenRefreshFailed` 错误类型
- [ ] 处理 `OAuthTokenExpired` 错误，触发 refresh
- [ ] 测试：正常 refresh 流程
- [ ] 测试：并发请求协调
- [ ] 测试：refresh 失败后的 fail-fast 行为
- [ ] 测试：优雅关闭

## 后续优化（可选）

1. **Token hash 验证**: 在触发 refresh 前验证 `token_hash` 是否匹配
2. **指标收集**: 记录 refresh 次数、失败次数、等待时间等
3. **主动 refresh**: 在 token 即将过期前主动刷新（需要 token expiry 信息）
