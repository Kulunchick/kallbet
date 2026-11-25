import pathlib,urllib.request as u, urllib.parse as q,urllib.error,json,time,gzip,sys
root=pathlib.Path("sidecar/tests/fixtures/backtest")
root.mkdir(exist_ok=True)
s={}
for line in pathlib.Path("D:/Projects/KallBet/.env").read_text(encoding="utf-8-sig").splitlines():
 k,_,v=line.partition("=")
 if k in ("ODDSPAPI_API_KEY","PANDASCORE_TOKEN"): s[k]=v.strip().strip('"').strip("'")
def get(url,params={},headers={}):
 try:
  b=u.urlopen(u.Request(url+"?"+q.urlencode(params),headers=headers),timeout=35).read()
 except urllib.error.HTTPError as e: raise RuntimeError(f"HTTP {e.code}") from None
 except Exception: raise RuntimeError("transport failure") from None
 if any(x and x.encode() in b for x in s.values()) and "/account" not in url: raise RuntimeError("credential in response")
 return b
def save(name,b,url):
 p=root/(name+".gz")
 p.write_bytes(gzip.compress(b,mtime=0))
 with (root/"sources.tsv").open("a",encoding="utf8") as f:f.write(p.name+"\t"+url+"\n")
def panda(path,params):
 url="https://api.pandascore.co"+path
 return get(url,params,{"Authorization":"Bearer "+s["PANDASCORE_TOKEN"]})
def odds(path,params):
 return get("https://api.oddspapi.io/v4/"+path,params|{"apiKey":s["ODDSPAPI_API_KEY"]})
def count():
 def walk(v):
  if isinstance(v,dict):
   for k,x in v.items():
    if k.lower().replace("_","")=="requestcount": print("request_count",x,flush=True)
    else:walk(x)
  elif isinstance(v,list):
   for x in v:walk(x)
 walk(json.loads(odds("account",{})))
if sys.argv[1]=="odds":
 count()
 for f in json.loads(pathlib.Path("sidecar/tests/fixtures/odds/oddspapi-fixtures-pgl-singapore-settled.json").read_text()):
  id=f["fixtureId"]
  if id=="od174000143389068" or not f["hasOdds"]:continue
  params={"fixtureId":id,"bookmakers":"pinnacle+30"}
  save(id+".json",odds("historical-odds",params),"https://api.oddspapi.io/v4/historical-odds?"+q.urlencode(params))
  time.sleep(5.2)
 count()
elif sys.argv[1]=="find":
 for dt in ["2024-03-17","2025-06-03","2025-11-24"]:
  arr=json.loads(panda("/csgo/series",{"range[begin_at]":dt+","+dt[:8]+str(int(dt[8:])+1).zfill(2),"per_page":100}))
  print([(i["id"],i["league"]["name"],i["full_name"]) for i in arr])
elif sys.argv[1]=="matches":
 for series in sys.argv[2:]:
  ts=json.loads(panda("/series/"+series+"/tournaments",{"per_page":100}))
  for t in ts:
   if series=="11176" and t["name"]!="Stage 1":continue
   url="/tournaments/"+str(t["id"])+"/matches"
   save("panda-"+str(t["id"])+".json",panda(url,{"per_page":100}),"https://api.pandascore.co"+url+"?per_page=100")
elif sys.argv[1]=="vrs":
 rev="f8c55f250fdef796d708aa32814587932154fa04"
 for d in ["2024_11_25","2024_12_02","2025_06_02","2025_11_03","2025_12_01","2026_06_01"]:
  name="standings_global_"+d+".md"
  url=f"https://raw.githubusercontent.com/ValveSoftware/counter-strike_regional_standings/{rev}/live/{d[:4]}/{name}"
  save(name,get(url),url)
elif sys.argv[1]=="history":
 count()
 for f in json.loads(pathlib.Path("sidecar/tests/fixtures/odds/oddspapi-fixtures-pgl-singapore-settled.json").read_text()):
  id=f["fixtureId"]
  if id=="od174000143389068":continue
  params={"fixtureId":id,"bookmakers":"pinnacle+30"}
  save(id+".json",odds("historical-odds",params),"https://api.oddspapi.io/v4/historical-odds?"+q.urlencode(params))
  time.sleep(5.2)
 count()
elif sys.argv[1]=="lp":
 from datetime import datetime,timedelta
 stems={7278:"PGL/2024/Copenhagen",8751:"Perfect World/Major/2024/Shanghai",9309:"BLAST/Major/2025/Austin",9822:"StarLadder/2025/Major",10488:"Intel Extreme Masters/2026/Cologne",11176:"PGL/2026/Singapore"}
 for path in sorted(root.glob("panda*")):
  matches=json.load(gzip.open(path)); first=min(m["begin_at"] for m in matches if m["status"]=="finished" and m["begin_at"])
  first=(datetime.fromisoformat(first)-timedelta(seconds=1)).isoformat().replace("+00:00","Z")
  m=matches[0]; stage=m["tournament"]["name"]; title=stems[m["serie_id"]]+"/"+stage
  params={"action":"query","prop":"revisions","rvprop":"ids|timestamp|content","rvslots":"main","rvlimit":1,"rvstart":first,"rvdir":"older","formatversion":2,"format":"json","titles":title}
  url="https://liquipedia.net/counterstrike/api.php"
  try:
   b=get(url,params,{"User-Agent":"KallBet/0.1 (https://kallbet.kulunchick.tech)","Accept-Encoding":"gzip"})
   if b[:2]==b"\x1f\x8b": b=gzip.decompress(b)
   if json.loads(b)["query"]["pages"][0].get("revisions"):save("lp-"+str(m["tournament_id"])+".json",b,url+"?"+q.urlencode(params))
  except Exception as e: print(type(e).__name__,title)
  time.sleep(2.1)
elif sys.argv[1]=="rest":
 params={"tournamentId":46101,"statusId":2,"from":"2026-06-02","to":"2026-06-22"}
 try:
  b=odds("fixtures",params); games=json.loads(b)
  print("Cologne candidates",len(games),flush=True)
  games=[g for g in games if "2026-06-02"<=g["startTime"]<"2026-06-22"]
  if games:save("cologne-fixtures.json",b,"https://api.oddspapi.io/v4/fixtures?"+q.urlencode(params))
 except RuntimeError as e: print("Cologne",e);games=[]
 games=json.loads(pathlib.Path("sidecar/tests/fixtures/odds/oddspapi-fixtures-pgl-singapore-settled.json").read_text())+games
 for f in games:
  id=f["fixtureId"]
  if id=="od174000143389068" or (root/(id+".json.gz")).exists(): continue
  params={"fixtureId":id,"bookmakers":"pinnacle+30"}
  try:save(id+".json",odds("historical-odds",params),"https://api.oddspapi.io/v4/historical-odds?"+q.urlencode(params))
  except RuntimeError as e:
   print(id,e,flush=True)
   if "404" not in str(e):break
  time.sleep(5.2)
 count()
elif sys.argv[1]=="revisions":
 from datetime import datetime,timedelta
 stages=json.loads(pathlib.Path("target/replay/stages.json").read_text())
 for stage in stages:
  if stage["id"] not in (13167,15479,22378,15477):continue
  if stage["id"] in (13167,15479):
   title={13167:"PGL/2024/Copenhagen",15479:"Perfect World/Major/2024/Shanghai"}[stage["id"]]
   requests=[("lp-"+str(stage["id"]),stage["cutoff"])]
  else:
   title={22378:"PGL/2026/Singapore",15477:"Perfect World/Major/2024/Shanghai"}[stage["id"]]
   rows=sorted(stage["matches"],key=lambda m:(m["game"]["scheduled_start"],m["game"]["id"]))
   requests=[("roster-"+str(m["game"]["id"]),min(datetime.fromisoformat(m["game"]["scheduled_start"])-timedelta(minutes=15),datetime.fromisoformat(m["actual"])).isoformat()) for m in (rows[:2] if stage["id"]==15477 else rows)]
  for name,cut in requests:
   until=(datetime.fromisoformat(cut)-timedelta(seconds=1)).isoformat().replace("+00:00","Z")
   params={"action":"query","prop":"revisions","rvprop":"ids|timestamp|content","rvslots":"main","rvlimit":1,"rvstart":until,"rvdir":"older","formatversion":2,"format":"json","titles":title}
   url="https://liquipedia.net/counterstrike/api.php"
   b=get(url,params,{"User-Agent":"KallBet/0.1 (https://kallbet.kulunchick.tech)","Accept-Encoding":"gzip"})
   if b[:2]==b"\x1f\x8b":b=gzip.decompress(b)
   if json.loads(b)["query"]["pages"][0].get("revisions"):save(name+".json",b,url+"?"+q.urlencode(params))
   time.sleep(2.1)
elif sys.argv[1]=="shanghai":
 params={"action":"query","prop":"revisions","rvprop":"ids|timestamp|content","rvslots":"main","rvlimit":1,"rvstart":"2024-12-12T06:09:52Z","rvdir":"older","formatversion":2,"format":"json","titles":"Perfect World/Major/2024/Shanghai/Playoff Stage"}
 url="https://liquipedia.net/counterstrike/api.php"
 b=get(url,params,{"User-Agent":"KallBet/0.1 (https://kallbet.kulunchick.tech)","Accept-Encoding":"gzip"})
 if b[:2]==b"\x1f\x8b":b=gzip.decompress(b)
 save("shanghai-playoff.json",b,url+"?"+q.urlencode(params))
