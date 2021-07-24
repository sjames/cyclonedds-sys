#include "dds/dds.h"
#include "dds/ddsi/ddsi_serdata.h"
#include "dds/ddsi/ddsi_sertype.h"

/*  dds_status_id_t is not used by any function so it doesn't turn up in the generated
    bindings. This dummy function forces the dds_status_id_t to be used.
 */

void _dummy(dds_status_id_t status); 
