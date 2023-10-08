from collections import defaultdict
f = open('opcodes.rs')
map = defaultdict(list)
for s in f:
    s = s.strip()
    if s[0:len("OpCode::new")] == "OpCode::new":
        s = (s[len("OpCode::new("):].split(","))
        map[s[1]].append(s[0])

for key, val in map.items():
    print("//", key.strip("  \"\""))
    print(' | '.join(val) + " => {},\n")

