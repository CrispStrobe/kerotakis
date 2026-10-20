def decode_plan_transport(value):
 import base64,hashlib,json,zlib
 maximum=16*1024*1024
 assert type(value) is dict and set(value)=={'encoding','decoded_bytes','decoded_sha256','payload'}
 assert value['encoding']=='base64-zlib-json-v1' and type(value['decoded_bytes']) is int and 0<value['decoded_bytes']<=maximum
 assert type(value['decoded_sha256']) is str and len(value['decoded_sha256'])==64 and all(c in '0123456789abcdef' for c in value['decoded_sha256'])
 assert type(value['payload']) is str and len(value['payload'])<=((maximum+2)//3)*4
 packed=base64.b64decode(value['payload'],validate=True)
 assert base64.b64encode(packed).decode('ascii')==value['payload'] and len(packed)<=maximum
 decoder=zlib.decompressobj()
 raw=decoder.decompress(packed,maximum+1)
 assert len(raw)==value['decoded_bytes'] and len(raw)<=maximum
 assert decoder.eof and not decoder.unused_data and not decoder.unconsumed_tail,'Truncated, concatenated or excess zlib transport'
 assert hashlib.sha256(raw).hexdigest()==value['decoded_sha256']
 def unique_pairs(pairs):
  result={}
  for key,item in pairs:
   assert key not in result,'Duplicate JSON plan key'
   result[key]=item
  return result
 def reject_constant(text):raise ValueError('Nonfinite JSON plan value: '+text)
 result=json.loads(raw.decode('utf-8'),object_pairs_hook=unique_pairs,parse_constant=reject_constant)
 assert type(result) is dict
 return result
