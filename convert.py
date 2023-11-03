string = "0"
ans = []
while(string != ""):
    string = input("waiting...")
    ans.append(string)
commands = []
for i in range(2):
    commands.append("self." + input("command ") + "(&opcode.mode);")

print(" | ".join(ans) + "=> {")

for i in commands:
    print (i)
print("}")