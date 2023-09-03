/// Write Flow
/// 1. Write the key-value pair to write-ahead log, so that it can be recovered after the storage engine crashes.
/// 2. Write the key-value pair to memtable.
/// After (1) and (2) completes, we can notify the user that the write operation is completed.
/// 3. When a memtable is full, we will flush it to the disk as an SST file in the background.
/// 4. We will compact some files in some level into lower levels to maintain a good shape for the LSM tree,
/// so that read amplification is low.
///
/// Read Flow
/// 1. We will first probe all the memtables from latest to oldest.
/// 2. If the key is not found, we will then search the entire LSM tree containing SSTs to find the data.

