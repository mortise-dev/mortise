import sys, importlib
sys.path.insert(0, '.')
import lib
mods = sys.argv[1:] or ['p_start', 'p_read', 'p_write', 'p_global', 'p_flows', 'p_ref', 'p_open', 'p_tokens', 'p_components', 'p_arch', 'p_index']
for m in mods:
    importlib.import_module(m).build()
lib.finish()
