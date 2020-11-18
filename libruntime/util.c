#include "libruntime.h"

void pregs_free_all(hashset_t set, size_t* v, void** allocs) {
  hashset_itr_t iter = hashset_iterator(set);

  while (hashset_iterator_has_next(iter) == 1) {
    hashset_iterator_next(iter);
    free(allocs[v[hashset_iterator_value(iter)]]);
  }
}