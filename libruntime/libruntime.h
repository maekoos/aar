#ifndef AAR_LIBRUNTIME_H
#define AAR_LIBRUNTIME_H

#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#include "hashset/hashset.h"
#include "hashset/hashset_itr.h"

bool NOT_EQUAL(int32_t, int32_t);
bool IS_EQUAL(int32_t, int32_t);

// void* MAKE_ARRAY(uint32_t, uint16_t);

// void pregs_free(hashset_t set, size_t reg, void* ptr);
void pregs_free_all(hashset_t set, size_t* v, void** allocs);

#endif