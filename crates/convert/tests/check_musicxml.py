"""Independent offline XSD oracle; requires python3-lxml."""
from pathlib import Path
import sys
from lxml import etree

schema_path = Path(__file__).resolve().parents[2] / 'research/standards/musicxml/musicxml-4.0/schema/musicxml.xsd'
class LocalSchema(etree.Resolver):
    def resolve(self, url, public_id, context):
        name = url.rsplit('/', 1)[-1]
        if name in ('xml.xsd', 'xlink.xsd'):
            return self.resolve_filename(str(schema_path.parent / name), context)

parser = etree.XMLParser(no_network=True, resolve_entities=False)
parser.resolvers.add(LocalSchema())
schema = etree.XMLSchema(etree.parse(str(schema_path), parser))
if sys.argv[1:] == ['--stdin-json']:
    import json
    for source in json.load(sys.stdin):
        schema.assertValid(etree.fromstring(source.encode('utf-8'), parser))
else:
    for path in sys.argv[1:]:
        schema.assertValid(etree.parse(path, parser))
