/* Stripped down selinux.h header with only the functions we need */

#include <stddef.h>

int setexeccon(const char *context);

int getexeccon(char **context);

int security_getenforce(void);

int security_load_policy(void *data, size_t len);