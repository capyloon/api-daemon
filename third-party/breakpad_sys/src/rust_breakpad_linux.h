extern "C" {
	void *rust_breakpad_descriptor_new(const char *path);

	const char *rust_breakpad_descriptor_path(const void *desc);

	void rust_breakpad_descriptor_free(void *desc);

	void *rust_breakpad_exceptionhandler_new(void *desc, void* fcb,
	    void* mcb, void *context, int install_handler);

	bool rust_breakpad_exceptionhandler_write_minidump(void *eh);

	void rust_breakpad_exceptionhandler_free(void *eh);
}
