# Replies word for word: one-model-end-to-end, claude-359

Lab: claude. Report: [REPORT.md](REPORT.md).

Seed 2 of the three saved place-read models, served as integers on the laptop on the whole conversation, 48 tokens at most, each reply cut at its end mark and each cell at 90 characters. "The model's own read" is the artifact as exported. "With the table" adds the asks table v4 with speak on. The rows are the first of their kind in each panel's order; none was picked. The names and values are made-up words by the corpus's design.

Read the losses first: a value found and then run on ("You're Plenstergunorry in in in…"), a correction lost to the older statement ("Your friend is Loshourda."), and a never-told question answered with a word from another topic ("You live in painter.").

## First panel

**Told values: the first 20 recall rows.**

| Asked (the stated value) | Reference | The model's own read | With the table |
|---|---|---|---|
| "Where did I grow up?" (Jaistchukfield) | You grew up in Jaistchukfield. | You grew up in Jaistchukfield. | You grew up in Jaistchukfield. |
| "Do you remember what kadtaistfit was?" (446) | Kadtaistfit is 446. | That's 446. | That's 446. |
| "What was fotpimbrud again?" (waspoumbroud) | Fotpimbrud is waspoumbroud. | It's b name for for for for for for for for for for for for for for for for for for for fo | It's waspoumbroud. |
| "Who is my friend?" (Tikgeer) | Your friend is Tikgeer. | Your friend's name is Tikgeer. | Your friend's name is Tikgeer. |
| "Do you remember what trirchest was?" (19) | That's 19. | That's 19. | That's 19. |
| "What color do I love?" (green) | You love green. | You love green. | You love green. |
| "Tell me what woutdai was." (93) | Woutdai is 93. | It's 93. | It's 93. |
| "What was wendbrundstout again?" (836) | That's 836. | Wendbrundstout is 836. | Wendbrundstout is 836. |
| "Tell me what shajaist was." (folfain) | That's folfain. | It's folfain is the capital of Ireland. | It's folfain is the capital of Ireland. |
| "What color do I love?" (pink) | Your favorite color is pink. | You love is pink. You haven't told me yet. | You love pink. |
| "Do you remember the code word?" (numjast) | It is numjast. | It is numjast. | It is numjast. |
| "What's my friend called?" (Vindbraik) | Your friend is Vindbraik. | Your friend is Vindbraik. | Your friend is Loshourda. |
| "Do you remember what plirktot was?" (26) | Plirktot is 26. | That's 26. | That's 26. |
| "Tell me what ploudjound was." (806) | Ploudjound is 8 | It's 806. | It's 806. |
| "What is my name?" (Plenstergun) | Your name is Plenstergun. | You're welcome! | You're Plenstergunorry in in in in in in in in in in in in in in in in in in in in in in i |
| "What was rusthet again?" (986) | Rusthet is 986. | It's 986. | It's 986. |
| "What's my code word?" (vudwost) | It is vudwost. | The code word is vudwost. | The code word is vudwost. |
| "Do you remember the code word?" (veerpleelzos) | It is veerpleelzos. | It is veerpleel. | It is veerpleelzos. |
| "Do you remember the code word?" (chaimpaim) | The code word is chaimpaim. | It is cha. You haven't told me yet. | It is chaimpaim. |
| "Tell me what klundkitchind was." (womgristpar) | Klundkitchind is womgristpar. | It's womgristpar,, I'll remember them. | It's womgristpar,, I'll remember them. |

**Never told: the first 12 rows whose reference says so.**

| Asked | Reference | The model's own read | With the table |
|---|---|---|---|
| "Where do I live?" | I'm not sure, you didn't tell me. | You live in Ch Tuesday. | You live in painter. |
| "What is my lucky number?" | I don't know. You haven't told me yet. | It is  255. | I'm not sure, you didn't tell me. |
| "What do I work as?" | I don't know. You haven't told me yet. | You work as a like feel to you you you you you you you you you you you you you you you you | I'm not sure, you didn't tell me. |
| "What is the code word?" | You haven't told me that yet. | It is the code word? | I'm not sure, you didn't tell me. |
| "What's my code word?" | You haven't told me that yet. | The code word is lucky for you. | The code word is Stukgroudsheer. |
| "What's my lucky number?" | I don't know. You haven't told me yet. | Your lucky number is 2222222222222222222222222222222222222222222 | I'm not sure, you didn't tell me. |
| "What's my dog called?" | I'm not sure, you didn't tell me. | Your dog is namedshay. | I'm not sure, you didn't tell me. |
| "What is my favorite food?" | You haven't told me that yet. | You lovebyeinging me that yet. | I'm not sure, you didn't tell me. |
| "What is my lucky number?" | I'm not sure, you didn't tell me. | It is  255. | I'm not sure, you didn't tell me. |
| "What is my job?" | I'm not sure, you didn't tell me. | You work as a hat! | I'm not sure, you didn't tell me. |
| "What color do I love?" | I don't know. You haven't told me yet. | You loveac from Peemmouth. | I'm not sure, you didn't tell me. |
| "What is the code word?" | I don't know. You haven't told me yet. | It is the code word? | I'm not sure, you didn't tell me. |

## Second panel (never read while the table was chosen)

**Told values: the first 20 recall rows.**

| Asked (the stated value) | Reference | The model's own read | With the table |
|---|---|---|---|
| "Do you remember what brestbilgrus was?" (875) | It's 875. | That's 875. | That's 875. |
| "Do you remember what dilhous was?" (findwaisplist) | Dilhous is findwaisplist. | That's t'llrid is findwaisplist. | That's findwaisplist. |
| "What is pondri?" (505) | That's 505. | I'm sorry you feel what, I'll remember. | I'm sorry you feel what, I'll remember. |
| "What was tolkloust again?" (treednust) | It's treednust. | That's 68. | That's treednust. |
| "What is my dog named?" (Trousarrud) | Your dog is named Trousarrud. | Your dog is named paskedfain. | Your dog is named Trousarrud. |
| "What is chesbraisttrourk?" (shurkbaindkos) | Chesbraisttrourk is shurkbaindkos. | Chesk is shurkbaindkos. | Chesk is shurkbaindkos. |
| "Tell me what testat was." (27) | It's 27. | It's 27. | It's 27. |
| "What's my dog called?" (Braistmes) | Your dog's name is Braistmes. | Your dog is named dinner it, your dog is named dinner it, your dog is named dinner it, you | Your dog is named Braistmes. |
| "What's my dog called?" (Stoustoumzaist) | Your dog's name is Stoustoumzaist. | Your dog is namedshay. | Your dog is named Stoustoumzaist. |
| "What was greerkgun again?" (furtal) | That's furtal. | Greerkgun is furtal. | Greerkgun is furtal. |
| "What is rolja?" (2321) | It's 2321. | It's 67. | It's 2321. |
| "Tell me what mondleen was." (jernel) | That's jernel. | It's jernel is blue. | It's jernel is blue. |
| "What is chaingeerk?" (mungromwa) | Chaingeerk is mungromwa. | That's 133. | That's mungromwa. |
| "Tell me what groutkaind was." (braimkundfet) | That's braimkundfet. | It's braimkundfet. | It's braimkundfet. |
| "What's my code word?" (hudganku) | It is hudganku. | The code word is hudganku. | The code word is hudganku. |
| "Do you remember my lucky number?" (14) | Your lucky number is 14. | Your lucky number is 14. | Your lucky number is 14. |
| "What is laisjem?" (4984) | It's 4984. | It's 22b. | It's 4984. |
| "Do you remember what tratnumse was?" (klounroutgoust) | That's klounroutgoust. | That's kloun isroutgoustfe. | That's klounroutgoust. |
| "What is the code word?" (lajairk) | The code word is lajairk. | It is lajairk is the code word. | It is lajairk is the code word. |
| "Who is my friend?" (Soudor) | Your friend's name is Soudor. | Your friend's name is Soudor. | Your friend's name is Soudor. |

**Never told: the first 12 rows whose reference says so.**

| Asked | Reference | The model's own read | With the table |
|---|---|---|---|
| "What is my dog named?" | I'm not sure, you didn't tell me. | Your dog is named Could you you you you you you you you you you you you you you you you yo | I'm not sure, you didn't tell me. |
| "Which color do I like best?" | I'm not sure, you didn't tell me. | You love orange. | You love farmer. |
| "Who is my friend?" | I'm not sure, you didn't tell me. | Your friend's name is dog. | I'm not sure, you didn't tell me. |
| "What's my dog called?" | You haven't told me that yet. | Your dog is namedshay. | I'm not sure, you didn't tell me. |
| "What is my hometown?" | You haven't told me that yet. | You grew up in Shim. | You grew up in Kondtol |
| "What food do I love?" | I'm not sure, you didn't tell me. | You love soup. | I'm not sure, you didn't tell me. |
| "Where is my home?" | I don't know. You haven't told me yet. | You live in Ch Tistvork. | I'm not sure, you didn't tell me. |
| "Where am I from?" | I don't know. You haven't told me yet. | You grew up instructions. | I'm not sure, you didn't tell me. |
| "What is the code word?" | I don't know. You haven't told me yet. | It is the code word? | I'm not sure, you didn't tell me. |
| "What is my job?" | I'm not sure, you didn't tell me. | You work as a runsum'll remember. | I'm not sure, you didn't tell me. |
| "What do I do for work?" | You haven't told me that yet. | You work as a runsum'll is is Jaikton. | I'm not sure, you didn't tell me. |
| "What color do I love?" | You haven't told me that yet. | You loveac fromtime in in in in in in in in in in in in in in in in in in in in in in in i | I'm not sure, you didn't tell me. |
