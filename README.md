# verified-wasm-runtime
To setup:  
`make boostrap`

To build rust code:  
`make build`

To verify:  
`make verify`

To rebuild the verifier after changing it's source code:  
`make prusti`



## Useful documentation
Interface for WASI calls: [WASI API](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md)  

## Syscalls

- [x] [args_get](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-args_getargv-pointerpointeru8-argv_buf-pointeru8---result-errno)
- [x] [args_sizes_get](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-args_sizes_get---resultsize-size-errno)
- [x] [environ_get](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-environ_getenviron-pointerpointeru8-environ_buf-pointeru8---result-errno)
- [x] [environ_sizes_get](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-environ_sizes_get---resultsize-size-errno)
- [x] [clock_res_get](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-clock_res_getid-clockid---resulttimestamp-errno)
- [x] [clock_time_get](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-clock_time_getid-clockid-precision-timestamp---resulttimestamp-errno)
- [x] [fd_advise](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-fd_advisefd-fd-offset-filesize-len-filesize-advice-advice---result-errno)
- [x] [fd_allocate](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-fd_allocatefd-fd-offset-filesize-len-filesize---result-errno)
- [x] [fd_close](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-fd_closefd-fd---result-errno)
- [x] [fd_datasync](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-fd_datasyncfd-fd---result-errno)
- [x] [fd_fdstat_get](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-fd_fdstat_getfd-fd---resultfdstat-errno)
- [x] [fd_fdstat_set_flags](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-fd_fdstat_set_flagsfd-fd-flags-fdflags---result-errno)
- [ ] [fd_fdstat_set_rights](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-fd_fdstat_set_rightsfd-fd-fs_rights_base-rights-fs_rights_inheriting-rights---result-errno)
- [x] [fd_filestat_get](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-fd_filestat_getfd-fd---resultfilestat-errno)
- [x] [fd_filestat_set_size](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-fd_filestat_set_sizefd-fd-size-filesize---result-errno)
- [x] [fd_filestat_set_times](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-fd_filestat_set_timesfd-fd-atim-timestamp-mtim-timestamp-fst_flags-fstflags---result-errno)
- [x] [fd_pread](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-fd_preadfd-fd-iovs-iovec_array-offset-filesize---resultsize-errno)
- [x] [fd_prestat_get](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-fd_prestat_getfd-fd---resultprestat-errno)
- [x] [fd_prestat_dir_name](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-fd_prestat_dir_namefd-fd-path-pointeru8-path_len-size---result-errno)
- [x] [fd_pwrite](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-fd_pwritefd-fd-iovs-ciovec_array-offset-filesize---resultsize-errno)
- [x] [fd_read](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-fd_readfd-fd-iovs-iovec_array---resultsize-errno)
- [x] [fd_readdir](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-fd_readdirfd-fd-buf-pointeru8-buf_len-size-cookie-dircookie---resultsize-errno)
- [x] [fd_renumber](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-fd_renumberfd-fd-to-fd---result-errno)
- [x] [fd_seek](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-fd_seekfd-fd-offset-filedelta-whence-whence---resultfilesize-errno)
- [x] [fd_sync](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-fd_syncfd-fd---result-errno)
- [x] [fd_tell](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-fd_tellfd-fd---resultfilesize-errno)
- [x] [fd_write](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-fd_writefd-fd-iovs-ciovec_array---resultsize-errno)
- [x] [path_create_directory](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-path_create_directoryfd-fd-path-string---result-errno)
- [x] [path_filestat_get](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-path_filestat_getfd-fd-flags-lookupflags-path-string---resultfilestat-errno)
- [x] [path_filestat_set_times](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-path_filestat_set_timesfd-fd-flags-lookupflags-path-string-atim-timestamp-mtim-timestamp-fst_flags-fstflags---result-errno)
- [x] [path_link](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-path_linkold_fd-fd-old_flags-lookupflags-old_path-string-new_fd-fd-new_path-string---result-errno)
- [x] [path_open](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-path_openfd-fd-dirflags-lookupflags-path-string-oflags-oflags-fs_rights_base-rights-fs_rights_inheriting-rights-fdflags-fdflags---resultfd-errno)
- [x] [path_readlink](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-path_readlinkfd-fd-path-string-buf-pointeru8-buf_len-size---resultsize-errno)
- [x] [path_remove_directory](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-path_remove_directoryfd-fd-path-string---result-errno)
- [x] [path_rename](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-path_renamefd-fd-old_path-string-new_fd-fd-new_path-string---result-errno)
- [x] [path_symlink](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-path_symlinkold_path-string-fd-fd-new_path-string---result-errno)
- [x] [path_unlink_file](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-path_unlink_filefd-fd-path-string---result-errno)
- [x] [poll_oneoff](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-poll_oneoffin-constpointersubscription-out-pointerevent-nsubscriptions-size---resultsize-errno)
- [x] [proc_exit](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-proc_exitrval-exitcode)
- [x] [proc_raise](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-proc_raisesig-signal---result-errno)
- [x] [sched_yield](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-sched_yield---result-errno)
- [x] [random_get](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-random_getbuf-pointeru8-buf_len-size---result-errno)
- [x] [sock_recv](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-sock_recvfd-fd-ri_data-iovec_array-ri_flags-riflags---resultsize-roflags-errno)
- [x] [sock_send](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-sock_sendfd-fd-si_data-ciovec_array-si_flags-siflags---resultsize-errno)
- [x] [sock_shutdown](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-sock_shutdownfd-fd-how-sdflags---result-errno)
- [x] socket (dynamically creates a sandbox)
- [x] sock_connect (like the connect call in posix)
