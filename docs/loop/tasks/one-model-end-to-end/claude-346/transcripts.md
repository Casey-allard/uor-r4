# claude-346: replies word for word

Float scoring pass on each job's last checkpoint: greedy, turn memory on, context 192, 48 reply tokens. A reply is cut at its end mark; one with none inside 48 tokens is marked. Made by `transcripts-346.sh` from the sealed scoring passes.

## Panel m, the first 12 prompts

### Prompt 1 (corpus position 52)

- wanted: ` wushodeer buldreestwend zeschai ploldrul graststast stoudhutlur ramdark`
- no address, seed 2: ` wushodeer buldreestwend zeschai ploldrul graststast stoudhutlur ramdark\nAssist`
- E8 address, seed 2: ` wushodededededededededededededededededededededededededededededededededededede`
- E8 address, seed 3: ` wushodeer buldreestwend zeschai ploldrul graststast stoudhutlur ramdark`
- token address, seed 2: ` wushodeer`

### Prompt 2 (corpus position 215)

- wanted: ` You grew up in Jaistchukfield.`
- no address, seed 2: ` You grew up in Jaistchukfield.`
- E8 address, seed 2: ` You grew up in Jaistchukfield.`
- E8 address, seed 3: ` You grew up in Jaistchukfield.`
- token address, seed 2: ` You grew up in Jaistchukfield.`

### Prompt 3 (corpus position 278)

- wanted: ` I am a small assistant. I can chat, remember what you tell me, and follow simple instructions.`
- no address, seed 2: ` I am a small assistant. I can chat, remember what you tell me, and follow simple instructions.`
- E8 address, seed 2: ` I am a small assistant. I can chat, remember what you tell me, and follow simple instructions.`
- E8 address, seed 3: ` I am a small assistant. I can chat, remember what you tell me, and follow simple instructions.`
- token address, seed 2: ` I am a small assistant. I can chat, remember what you tell me, and follow simple instructions.`

### Prompt 4 (corpus position 384)

- wanted: ` Kadtaistfit is 446.`
- no address, seed 2: ` That's 446.`
- E8 address, seed 2: ` That's 446.`
- E8 address, seed 3: ` That's 446.`
- token address, seed 2: ` That's 446.`

### Prompt 5 (corpus position 517)

- wanted: ` Fotpimbrud is waspoumbroud.`
- no address, seed 2: ` It's b name for for for for for for for for for for for for for for for for for for for for for for for for for for for for for for for for for for for for for for for for for for for for [no end in 48 tokens]`
- E8 address, seed 2: ` It's b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b [no end in 48 tokens]`
- E8 address, seed 3: ` It's b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b b [no end in 48 tokens]`
- token address, seed 2: ` It's b-aa-aa-aa-aa-aa-aa-aa-aa-aa-aa-aa-aa-aa-aa-aa [no end in 48 tokens]`

### Prompt 6 (corpus position 615)

- wanted: ` Your friend is Tikgeer.`
- no address, seed 2: ` Your friend's name is Tikgeer.`
- E8 address, seed 2: ` Your friend's name is Tikgeer.`
- E8 address, seed 3: ` Your friend's name is Tikgeer.`
- token address, seed 2: ` Your friend's name is Tikgeer.`

### Prompt 7 (corpus position 672)

- wanted: ` Here are three colors: brown, yellow, pink.`
- no address, seed 2: ` Here are three colors: orange, tasty!`
- E8 address, seed 2: ` Here are three colors: has six legs.`
- E8 address, seed 3: ` Here are three colors: Here are three colors: after after after after after after after after after after after after after after after after after after after after after after after after after after after after after after after after after after after after [no end in 48 tokens]`
- token address, seed 2: ` Here are three colors: pink, 7.`

### Prompt 8 (corpus position 736)

- wanted: ` vondhourk wourkzourk kendstoun pandplainhod sheennol ploundplaikvair`
- no address, seed 2: ` vondhourk kendstoun pandplainhod sheennol`
- E8 address, seed 2: ` vondhourk kendstoun pandplainhod sheennol`
- E8 address, seed 3: ` vondhourk kendstoun pandplainhod sheennol`
- token address, seed 2: ` vondhourk kendstoun pandplainhod sheennol`

### Prompt 9 (corpus position 947)

- wanted: ` A horse says neigh.`
- no address, seed 2: ` Neigh! That's the sound a horse makes.`
- E8 address, seed 2: ` Neigh! That's the sound a horse makes.`
- E8 address, seed 3: ` Neigh! That's the sound a horse makes.`
- token address, seed 2: ` Neigh! That's the sound a horse makes.`

### Prompt 10 (corpus position 1002)

- wanted: ` That's 19.`
- no address, seed 2: ` That's great news about the guitar!`
- E8 address, seed 2: ` That's great news about the guitar!`
- E8 address, seed 3: ` That's great news about the guitar!`
- token address, seed 2: ` That's great news about the guitar!`

### Prompt 11 (corpus position 1073)

- wanted: ` Oh, a scooter! How exciting.`
- no address, seed 2: ` That's great news about the watch!`
- E8 address, seed 2: ` That's great news about the book!`
- E8 address, seed 3: ` That's great news about the plant!`
- token address, seed 2: ` That's great news about the plant!`

### Prompt 12 (corpus position 1212)

- wanted: ` You love green.`
- no address, seed 2: ` You love green.`
- E8 address, seed 2: ` You love?`
- E8 address, seed 3: ` You love bread.`
- token address, seed 2: ` You love green.`


## Real text (OASST2 held out), the first 20 prompts

Each line is cut at 200 characters.

### No address (o-b)

1. user: `"Why Aristotelian view of physics (impetus and stuff) is wrong?"`
   - wanted: `"Aristotle's views on physics, which included the concept of impetus, were considered groundbreaking in his time, but have since been dis"`
   - said: `"towardsly of phys ( of\nAssistant:\nAssistant:\nAssistant:\nalshing, adbits ( of\nal use popular getsindc L in anyy"`
2. user: `"I'm having trouble understanding infinity. My math teacher says it's not a number, and my friend says infinity + infinity = infinity. I'm just really confused at the moment. I don't get why this is, `
   - wanted: `"Sure, i will explain it in general and then more precisely.\n\nInfinity is a concept for something endless.\nIt roughly means, that you can't \"dire"`
   - said: `"The You impual ofrop\n. I don't get why this is, would you be if!)riinationfinity. I don't get why this is, would you be if!)riinationfinity."`
3. user: `"Year the brand was established."`
   - wanted: `"I am unable to access real-time information or generate live data, and creating a comprehensive list of the top 100 global brands would requ"`
   - said: `"B(iv the brand was established.\n\n.\n.\n.\n.\n.\n.\n.\n.\n.\n.\n.\n\n.\n.\nal-'re Le been forgot"`
4. user: `"In TypeScript, how can I tell the type system \"I know that in theory this value may have these multiple types, but in practice I know it will only have this one\""`
   - wanted: `"TypeScript allows you to use union types based on several existing types (called member types). For example,"`
   - said: `"In Tyice I know it willstem \"I know it willstem \"I know it willstem \"I know it willstem \"I know it willstem \"I know it willstem \"I know"`
5. user: `"How can I do this in Javascript?"`
   - wanted: `"In JavaScript, you can't directly specify a variable's type in the same way you can in TypeScript. However, you can a"`
   - said: `"Dly ice still retun o sky0 and `."`
6. user: `"Can you tell me a bit about what has gone into your creation?"`
   - wanted: `"Sure! I was created using the latest in natural language processing technology, which enables me to understand and process natural language"`
   - said: `"Q: What did the otter say when he saw a plate of cookies? \nA: \"Otterly delicious!\""`
7. user: `"What is the family guy pipeline incident on tiktok?"`
   - wanted: `"The \"Family Guy pipeline incident\" is a recent viral trend that originated on TikTok. In these videos, users take a clip from the popular"`
   - said: `"Assistant:\nAssistant:\nAssistant:\nAssistant:\n alsM Ifedatesated aav notl is the family guy pipeline incident"`
8. user: `"Why does catnip make my cat go crazy?"`
   - wanted: `"The organic compound in catnip, nepetalactone, is what gives cats these bizarre reactions. Once the nepetalactone enters the catâĢĻs"`
   - said: `"Assistant:\nAssistant:\nAssistant:\nAssistant:\nAssistant:\nAssistant:\nAssistant:\nAssistant:"`
9. user: `"Answer all of my subsequent questions like a mad raving witch"`
   - wanted: `"Hear me, mortal!\n\nYour questions shall be answered with the fury of a thousand storms!\n\nSpeak your queries, if you dare to face the unle"`
   - said: `"Assistant:\nAssistant:\nAssistant:\nAssistant:\nAssistant:\nAssistant:\nAssistant:\nAssistant:"`
10. user: `"What is disodium guanylate and what is it used for?"`
   - wanted: `"Disodium guanylate is a type of sodium salt that is commonly used as a flavor enhancer in food and beverage products"`
   - said: `"The forAssistant: The forAssistant: The forAssistant: The forAssistant: The forAssistant: The forAssistant:\nAssistant:"`
11. user: `"I'm creating new pet collar with embedded GPS tracking - can you help me come up with a name for the product?"`
   - wanted: `"Sure! Here are a few suggestions for the name of your GPS pet collar:\n\n1. TrackPet\n2. SafeTail\n3. PawTrack\n4"`
   - said: `"Assistant:\nAssistant:\nAssistant:\nAssistant:\njThe,ows7 recri tricks iceventroutic spread threeusts manort thaty"`
12. user: `"Generate the lyrics to a song that begins like this \"Hey hey let's go...\""`
   - wanted: `"Hey, hey, let's go\nHey, hey, let's get up and rock and roll\nHey, hey, let's go\nHey, hey, let's get up and rock and roll"`
   - said: `"Generate the lys to a p twated inbitys to a p twated inbitys to ase the lys to ase the lys to a can..\"\nAss"`
13. user: `"Can you create some guitar tabs that would go well with this song?"`
   - wanted: `"Here's a simple guitar tab that should complement the upbeat, rock and roll vibe of your song. This tab is in the key of A major and consists of"`
   - said: `"toultUser: toultUser: toultUser: toultUser: tover. grassater app that would go well with this song?\n) InThere conop.\n) InThere conop."`
14. user: `"Could you put in laymen terms the terms of service for the open assistant project?"`
   - wanted: `"Ill try,\n\nThese are the terms of use for a website called LAION, which lets users create data to train an AI called Open Ass"`
   - said: `"Could you thisizedility the open assas a not\nAssistant: Could you thisizedility the anyone in laymsmsmsmsmsms andes`. A"`
15. user: `"What are the central beliefs in the civil religion that dominates the West at the beginning of the 21st century?"`
   - wanted: `"Civil religion is a set of beliefs, symbols, and rituals that give meaning and purpose to a nation or society. The West, in general"`
   - said: `"Onultackedjectbers aationain Le to your encast'sful andimles has to your encast's en caseral beliefs in the 21 asecsethn aation it mo gr dan"`
16. user: `"Tell me about climate change in a comprehensive and accurate manner, drawing from the most up-to-date information and sources."`
   - wanted: `"Climate change refers to long-term shifts in average temperatures, precipitation, wind patterns and other measures of climate that occur over"`
   - said: `"ority\"ing after fastert musicol ofbelis spr musicol ofbelis spr musicol ofbelis spr musicol ofbelis spr musicol ofbelis spr musicol ofbelis"`
17. user: `"I'm not sure if that is correct. Can you site your sources?"`
   - wanted: `"[The Unicode changelog](https://www.unicode.org/versions/Unicode5.1.0"`
   - said: `"Certainly! Let me break it down for you.\n\n1. The st entllingFromom one shapessistant: Certainly! Let me break it down for you. The st entl"`
18. user: `"Are these all the countries, if not please elaborate and include more countries but if so please tell me where the name of origin of these countries came from. Thank you!"`
   - wanted: `"Yes, those are all the countries on the continent of South America that start with \"C\" and \"P.\"\n\nThe origin of each country's name"`
   - said: `"American countries came from.c would Feruay"`
19. user: `"explain the web 2.0 and the reduction of the internet to a few main sites that everyone uses as if you are captain picard talking about the errors of the ways of the 21st centuary"`
   - wanted: `"Captain's log, stardate 2423.04.22. As I navigate the digital landscape of the 21st century, I can't help"`
   - said: `"vIry-ctortain picard talking about the 21st centuary\nAssistant: vIry-ctortain picard talking about the 21st centuary\nA"`
20. user: `"And how does Captain Kirk suggest we preserve the openness and diversity of the internet to benefit future generations, and what impact would it have on him and his crew?"`
   - wanted: `"Stardate 9704.22, Captain's Log\n\nAs we venture forth into the uncharted territories of the internet"`
   - said: `"ome rareameit future generations, and his crew?\nAssistant:ome rareameit future generations, and his crew?\nAssistant:ome rareameit"`

### E8 address (o-e8)

1. user: `"Why Aristotelian view of physics (impetus and stuff) is wrong?"`
   - wanted: `"Aristotle's views on physics, which included the concept of impetus, were considered groundbreaking in his time, but have since been dis"`
   - said: `"raityth che order9ain foodit cont his5 are careaverristant:Ptis ( of,ogren balen needm toagrindotelian view of,ogre"`
2. user: `"I'm having trouble understanding infinity. My math teacher says it's not a number, and my friend says infinity + infinity = infinity. I'm just really confused at the moment. I don't get why this is, `
   - wanted: `"Sure, i will explain it in general and then more precisely.\n\nInfinity is a concept for something endless.\nIt roughly means, that you can't \"dire"`
   - said: `"if and the moment. I don't get why this is, would you be able to me please?\nAssistant:if and the moment. I onesalsunork sarsunork sarsunork"`
3. user: `"Year the brand was established."`
   - wanted: `"I am unable to access real-time information or generate live data, and creating a comprehensive list of the top 100 global brands would requ"`
   - said: `"exhethe aticys andstablished.\nAssistant: exhethe aticys andstablished.\nAssistant: exhethe aticys andstablished."`
4. user: `"In TypeScript, how can I tell the type system \"I know that in theory this value may have these multiple types, but in practice I know it will only have this one\""`
   - wanted: `"TypeScript allows you to use union types based on several existing types (called member types). For example,"`
   - said: `"In Ty aark or, re*ionr leaves you of whochic sound: In Ty a. a. a. a. a. a. a. a. a. a. a. a. a."`
5. user: `"How can I do this in Javascript?"`
   - wanted: `"In JavaScript, you can't directly specify a variable's type in the same way you can in TypeScript. However, you can a"`
   - said: `"How can I thation andooseure SPering man to mooved at compile time, but you."`
6. user: `"Can you tell me a bit about what has gone into your creation?"`
   - wanted: `"Sure! I was created using the latest in natural language processing technology, which enables me to understand and process natural language"`
   - said: `"Q: What did the otter say when he saw a plate of cookies? \nA: \"Otterly delicious!\""`
7. user: `"What is the family guy pipeline incident on tiktok?"`
   - wanted: `"The \"Family Guy pipeline incident\" is a recent viral trend that originated on TikTok. In these videos, users take a clip from the popular"`
   - said: `"Theible pot W of, you dis you dis you dis you dis you dis you dis you dis you dis you dis you dis you dis you dis you dis you dis you dis you dis you dis you dis you dis you dis you dis"`
8. user: `"Why does catnip make my cat go crazy?"`
   - wanted: `"The organic compound in catnip, nepetalactone, is what gives cats these bizarre reactions. Once the nepetalactone enters the catâĢĻs"`
   - said: `"., theip make my cat go crazy?\nAssistant:., theip make my cat go crazy?\nAssistant:., theip make my cat go crazy?\nAssistant:"`
9. user: `"Answer all of my subsequent questions like a mad raving witch"`
   - wanted: `"Hear me, mortal!\n\nYour questions shall be answered with the fury of a thousand storms!\n\nSpeak your queries, if you dare to face the unle"`
   - said: `"Answer all of the, pot They designectoundhet Andle of the, thisuper alsoation1itb capr folder 1itb capr folder 1itb capr folder 1itb cap"`
10. user: `"What is disodium guanylate and what is it used for?"`
   - wanted: `"Disodium guanylate is a type of sodium salt that is commonly used as a flavor enhancer in food and beverage products"`
   - said: `"a Assistant: a Assistant: a Assistant: a Assistant: a Assistant: a Assistant: a Assistant: a Assistant:"`
11. user: `"I'm creating new pet collar with embedded GPS tracking - can you help me come up with a name for the product?"`
   - wanted: `"Sure! Here are a few suggestions for the name of your GPS pet collar:\n\n1. TrackPet\n2. SafeTail\n3. PawTrack\n4"`
   - said: `"Assistant:\nAssistant:\nAssistant: TheusI\nAssistant: TheusI\nAssistant: TheusI\nsc used clduct te."`
12. user: `"Generate the lyrics to a song that begins like this \"Hey hey let's go...\""`
   - wanted: `"Hey, hey, let's go\nHey, hey, let's get up and rock and roll\nHey, hey, let's go\nHey, hey, let's get up and rock and roll"`
   - said: `"re gooditeady| e byoreav m makew contin\"_ense\nAssistant: re gooditeady| e byppering. Sphert in the ly let's go. Sphert in the"`
13. user: `"Can you create some guitar tabs that would go well with this song?"`
   - wanted: `"Here's a simple guitar tab that should complement the upbeat, rock and roll vibe of your song. This tab is in the key of A major and consists of"`
   - said: `"Sion digasOK to inclu humat mott or its \"ge these ses ground th`esterednipy that would go well with gy that would go well with gy that would go well with gy"`
14. user: `"Could you put in laymen terms the terms of service for the open assistant project?"`
   - wanted: `"Ill try,\n\nThese are the terms of use for a website called LAION, which lets users create data to train an AI called Open Ass"`
   - said: `"Could you put in laymsistant: Could you put in laymsistant: Could you put in laymsistant: Could you put inatâAssistant: Could you"`
15. user: `"What are the central beliefs in the civil religion that dominates the West at the beginning of the 21st century?"`
   - wanted: `"Civil religion is a set of beliefs, symbols, and rituals that give meaning and purpose to a nation or society. The West, in general"`
   - said: `"ttornd\nAssistant:tor. it lost signog} of the 21st century?\nAssistant:tor. it lost signog} of the 21"`
16. user: `"Tell me about climate change in a comprehensive and accurate manner, drawing from the most up-to-date information and sources."`
   - wanted: `"Climate change refers to long-term shifts in average temperatures, precipitation, wind patterns and other measures of climate that occur over"`
   - said: `"Tell me about climate information and wrould off visitalroI wholeity can to H information and O agabanay specosct bs.\nAssistant: Tell"`
17. user: `"I'm not sure if that is correct. Can you site your sources?"`
   - wanted: `"[The Unicode changelog](https://www.unicode.org/versions/Unicode5.1.0"`
   - said: `"Certainly! Let me break it down for you.\n\n1. The st, the ligature that is: `(X Assistant: (X Assistant: (W"`
18. user: `"Are these all the countries, if not please elaborate and include more countries but if so please tell me where the name of origin of these countries came from. Thank you!"`
   - wanted: `"Yes, those are all the countries on the continent of South America that start with \"C\" and \"P.\"\n\nThe origin of each country's name"`
   - said: `"American countries came from. It pers moationeners can presentakutledentlewy no pr lose to imp R. It pers moationeners can presentakutledentlewy no pr"`
19. user: `"explain the web 2.0 and the reduction of the internet to a few main sites that everyone uses as if you are captain picard talking about the errors of the ways of the 21st centuary"`
   - wanted: `"Captain's log, stardate 2423.04.22. As I navigate the digital landscape of the 21st century, I can't help"`
   - said: `"explain the 21st centuary\nAssistant: explain the 21st centuary\nAssistant: explain the 21st centuary\n1st cent on"`
20. user: `"And how does Captain Kirk suggest we preserve the openness and diversity of the internet to benefit future generations, and what impact would it have on him and his crew?"`
   - wanted: `"Stardate 9704.22, Captain's Log\n\nAs we venture forth into the uncharted territories of the internet"`
   - said: `"a A pot."`
