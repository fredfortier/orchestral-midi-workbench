// Offline acceptance checks for the research/specification artifacts, not a converter.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync, mkdtempSync, rmSync, existsSync, readdirSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import path from 'node:path';
import os from 'node:os';
import { fileURLToPath, pathToFileURL } from 'node:url';

const converter = fileURLToPath(new URL('../', import.meta.url));
const research = path.resolve(converter, '../../.research');
const architecturePath = path.resolve(converter, '../../docs/architecture/convert.md');
const architecture = readFileSync(architecturePath, 'utf8');
const catalog = JSON.parse(readFileSync(path.join(converter,'mapping/orchestral-subclasses.json')));
const records = JSON.parse(readFileSync(path.join(research,'sources.json'))).records;
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const counts = [11,16,10,14,9,14,16,16]; // M2-123 Tables 8–15, not Appendix A.
assert.equal(catalog.rows.length,106);
assert.deepEqual(catalog.profileId,[0x7e,0x21,1,1,1]);
assert.equal(new Set(catalog.rows.map(r=>`${r.at}/${r.sub}`)).size,106);
assert.equal(new Set(catalog.rows.map(r=>r.id)).size,106);
assert.equal(new Set(catalog.rows.map(r=>r.term)).size,106);
for(let i=0;i<8;i++) assert.deepEqual(catalog.rows.filter(r=>r.at===16+i).map(r=>r.sub),
  Array.from({length:counts[i]},(_,j)=>j));
const native = new Set([...architecture.matchAll(/^\| (N\d+) \|/gm)].map(m=>m[1]));
assert.equal(native.size,26);
for(const row of catalog.rows) {
  assert(native.has(row.rule) || ['T','P','G1','G2'].includes(row.rule));
  assert(!/[\n|]/.test(row.term));
  assert.equal(row.term,row.term.normalize('NFC').trim().toLowerCase());
}
assert.equal(catalog.rows.find(r=>r.at===0x14 && r.sub===7).id,'color.lontano');
assert.equal(catalog.rows.find(r=>r.at===0x14 && r.sub===8).id,'color.singing');
assert.equal(catalog.rows.find(r=>r.at===0x16 && r.sub===15).id,'run.other');
const offSection = architecture.split('### Note Off subclasses')[1].split('### Controller encoding')[0];
assert.deepEqual([...offSection.matchAll(/^\| `10\/([0-4])` \|/gm)].map(m=>Number(m[1])),[0,1,2,3,4]);

for(const record of records) {
  const bytes=readFileSync(path.join(research,record.path));
  assert.equal(bytes.length,record.bytes,record.path);
  assert.equal(sha(bytes),record.sha256,record.path);
  if(record.derivedPath) assert.equal(sha(readFileSync(path.join(research,record.derivedPath))),
    record.derivedSha256,record.derivedPath);
  if(record.readablePath) assert.equal(sha(readFileSync(path.join(research,record.readablePath))),
    record.readableSha256,record.readablePath);
  if(record.offlinePath) {
    const offline=readFileSync(path.join(research,record.offlinePath));
    assert.equal(sha(offline),record.offlineSha256,record.offlinePath);
    for(const match of offline.toString().matchAll(/<img\b[^>]+src="([^"]+)"/g))
      assert(existsSync(path.resolve(research,path.dirname(record.offlinePath),match[1])));
  }
}
const berlin=records.filter(r=>r.groups?.includes('berlin'));
assert.equal(berlin.length,30);
assert.equal(records.filter(r=>r.kind==='manual').length,90);
assert.equal(records.filter(r=>r.kind==='manual-figure').length,101);
assert.equal(records.filter(r=>r.kind==='standard').length,12);
const audit=readFileSync(path.join(research,'BERLIN-AUDIT.md'),'utf8');
for(const source of berlin) assert(audit.includes(path.basename(source.path,'.html')+'.txt'),source.title);
const rows=JSON.parse(readFileSync(path.join(research,'berlin-manual-rows.json')));
assert.equal(rows.length,3636);
assert.deepEqual(new Set(rows.map(r=>r.source)),new Set(berlin.map(r=>r.path)));
const figures=new Set();
for(const source of records.filter(r=>r.kind==='manual')) {
  const html=readFileSync(path.join(research,source.path),'utf8');
  const body=html.match(/<article\s+id="fullArticle">([\s\S]*?)<\/article>/)[1];
  for(const match of body.matchAll(/<img\b[^>]+src="([^"]+)"/g))
    figures.add(new URL(match[1].replaceAll('&amp;','&'),source.url).href);
}
assert.deepEqual(figures,new Set(records.filter(r=>r.kind==='manual-figure').map(r=>r.url)));

const glyphs=JSON.parse(readFileSync(path.join(research,'standards/smufl/glyphnames-v1.4.json')));
const glyphSection=architecture.split('### Verified SMuFL identities')[1].split('### Pattern constraints')[0];
for(const match of glyphSection.matchAll(/`((?:strings|plucked|artic)[A-Za-z0-9]+)`/g))
  assert(Object.hasOwn(glyphs,match[1]),'Not a SMuFL 1.4 name: '+match[1]);
for(const file of ['standards/musicxml/musicxml-v4.0.zip','standards/smufl/smufl-v1.4.zip'])
  execFileSync('unzip',['-t',path.join(research,file)],{stdio:'pipe',maxBuffer:4*1024*1024});
const schemaRoot=path.join(research,'standards/musicxml/musicxml-4.0/schema');
for(const file of readdirSync(schemaRoot)) {
  const full=path.join(schemaRoot,file);
  if(!/\.(xsd|xsl|xml|dtd|mod|ent)$/.test(file)) continue;
  const original=execFileSync('unzip',['-p',path.join(research,'standards/musicxml/musicxml-v4.0.zip'),'musicxml-4.0/schema/'+file]);
  assert.equal(sha(readFileSync(full)),sha(original),'Changed extracted schema: '+file);
}

const temporary=mkdtempSync(path.join(os.tmpdir(),'midi2-spec-check-'));
try {
  const xmlEscape=s=>s.replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;');
  const fragment=`<note id="blurred">
  <pitch><step>D</step><octave>5</octave></pitch>
  <duration>1</duration>
  <notations><articulations><spiccato/></articulations></notations>
  <play>
    <other-play type="urn:midi2workbench:articulation:v2">{"technique":"short.spiccato","qualifiers":{"urn:midi2workbench:ot-berlin:sample-character":"blurred"},"binding":"example-berlin-strings-violin-i-v1"}</other-play>
  </play>
</note>`;
  const payload=JSON.parse(fragment.match(/<other-play[^>]*>([^<]+)<\/other-play>/)[1]);
  assert.equal(payload.technique,'short.spiccato');
  assert.equal(payload.qualifiers['urn:midi2workbench:ot-berlin:sample-character'],'blurred');
  writeFileSync(path.join(temporary,'annotation.json'),JSON.stringify(payload));
  const notes=catalog.rows.map((r,i)=>`<note id="catalog-${i}"><pitch><step>C</step><octave>4</octave></pitch><duration>1</duration><notations><technical><other-technical>${xmlEscape(r.term)}</other-technical></technical></notations></note>`).join('');
  const score='<score-partwise version="4.0"><part-list><score-part id="P1"><part-name>Example</part-name></score-part></part-list><part id="P1"><measure number="1"><attributes><divisions>1</divisions></attributes>'+fragment+notes+'</measure></part></score-partwise>';
  writeFileSync(path.join(temporary,'examples.musicxml'),score);
  const imports=[['http://www.musicxml.org/xsd/xml.xsd','xml.xsd'],['http://www.musicxml.org/xsd/xlink.xsd','xlink.xsd']];
  writeFileSync(path.join(temporary,'catalog.xml'),'<catalog xmlns="urn:oasis:names:tc:entity:xmlns:xml:catalog">'+imports.map(([id,file])=>`<system systemId="${id}" uri="${pathToFileURL(path.join(schemaRoot,file)).href}"/>`).join('')+'</catalog>');
  execFileSync('xmllint',['--nonet','--noout','--schema',path.join(schemaRoot,'musicxml.xsd'),path.join(temporary,'examples.musicxml')],{
    env:{...process.env,XML_CATALOG_FILES:path.join(temporary,'catalog.xml')},stdio:'pipe'
  });
} finally { rmSync(temporary,{recursive:true,force:true}); }

const on0=(0x40000000|(9<<20)|(74<<8)|0x11)>>>0;
const data=(10<<12)|(1<<6)|(1<<3)|2;
assert.equal(on0,0x40904a11); assert.equal(data,0xa04a);
assert.equal(((0x9000<<16)|data)>>>0,0x9000a04a);
for(let sub=0;sub<=4;sub++) for(let string=0;string<=7;string++)
  assert.equal(((sub<<12)|string)&0x0ff8,0);
assert.equal(60.5*2**25,0x79000000);
function upscale(value,from,to) {
  const shift=BigInt(to-from),v=BigInt(value);
  let result=v<<shift;
  if(value<=2**(from-1)) return Number(result);
  const repeatBits=BigInt(from-1);
  let repeat=v&((1n<<repeatBits)-1n);
  repeat=shift>repeatBits?repeat<<(shift-repeatBits):repeat>>(repeatBits-shift);
  while(repeat){result|=repeat;repeat>>=repeatBits;}
  return Number(result);
}
for(const [v,expected] of [[0,0],[5,0x0a00],[30,0x3c00],[64,0x8000],[70,0x8c30],[96,0xc104],[120,0xf1c7],[127,0xffff]])
  assert.equal(upscale(v,7,16),expected); // M2-115 Table 5.
assert.equal(upscale(64,7,32),0x80000000);
assert.equal(upscale(96,7,32),0xc1041041);
assert.equal(Math.max(1,0x0001>>9),1); // M2-104 D.2.1 Note On exception.

const binding=JSON.parse(readFileSync(path.join(converter,'examples/berlin-receiver.example.json')));
assert.equal(binding.status,'example-unverified');
assert.equal(binding.id,'example-berlin-strings-violin-i-v1');
function checkPeNames(value) {
  if(Array.isArray(value)) {value.forEach(checkPeNames);return;}
  if(value===null || typeof value!=='object')return;
  for(const [key,child] of Object.entries(value)) {
    assert(/^[a-z][A-Za-z0-9]*$/.test(key),'Illegal PE property name: '+key);
    checkPeNames(child);
  }
}
checkPeNames(binding);
for(const entry of binding.entries) {
  assert(catalog.rows.some(r=>r.id===entry.descriptor.technique));
  assert.equal(new Set(entry.descriptor.qualifiers.map(q=>q.id)).size,entry.descriptor.qualifiers.length);
  entry.descriptor.qualifiers.forEach(q=>JSON.parse(q.valueJson));
  const {at,sub,variation}=entry.wire;
  assert(at>=0x1a || catalog.rows.some(r=>r.at===at&&r.sub===sub));
  assert(variation>=0 && variation<16);
}
const blurred=binding.entries.find(e=>e.descriptor.qualifiers.length);
assert.deepEqual(blurred.wire,{at:0x1a,sub:0,variation:0});
assert.equal(JSON.parse(blurred.descriptor.qualifiers[0].valueJson),'blurred');
const resource=JSON.parse(readFileSync(path.join(converter,'examples/resource-list.example.json')))[0];
assert.equal(resource.resource,'X-ArticulationMap');
assert.equal(resource.schema.$ref,'midi+jsonschema://midi2workbenchArticulationMapV2');

const archiveReadme = path.join(research, 'README.md');
for(const file of [architecturePath, archiveReadme, path.join(research,'BERLIN-AUDIT.md')]) {
  for(const match of readFileSync(file,'utf8').matchAll(/\]\(([^)]+)\)/g)) {
    const target=match[1].split('#')[0];
    if(!target || /^[a-z][a-z0-9+.-]*:/i.test(target))continue;
    // The immutable archive introduction links to the pre-migration converter contract.
    const destination = file === archiveReadme && target === '../convert/README.md'
      ? architecturePath
      : path.resolve(path.dirname(file), target);
    assert(existsSync(destination),`${file}: broken link ${target}`);
  }
}
console.log('PASS: 106 named coordinates, 26 native predicates, release catalog, actual SMuFL names.');
console.log(`PASS: ${records.length} archived source records, 30 Berlin manuals, 3,636 table rows, all 101 figures.`);
console.log('PASS: archive/schema integrity, MusicXML examples for all named terms, local links, wire and scaling examples.');
console.log('Scope: documentation checks only; no conversion engine or live Berlin playback is asserted.');
