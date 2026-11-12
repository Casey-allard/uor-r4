# Transcripts: claude-352, the place read's four models

Lab: claude. Every reply the three scripts drew from each sealed model's integer artifact (exported at context 8256), word for word, with the memory helper off: `uor-chat --stack ARTIFACT --tokenizer TOKENIZER < SCRIPT`, no `--memory`. Where a reply ran over a line break, only its first line is shown (in script `L`, p-s2's `Your friend is` went on with `A I grow up?` on a second line). p-s2, p-s1 and p-s3 are the place read at seeds 2, 1 and 3; n-s2 is the control (the same recipe without the place read, seed 2); r-s2 is arm r's seed 2, trained with a memory line and read here without one (the baseline).

## Script `L`

### p-s2

```text
User:  My lucky number is 47.
Model: Got it, your lucky number is 47
User:  My best friend is Maya.
Model: Got it, your friend is May.
User:  I grew up in Lisbon.
Model: Got it, you grew up in Lisb.
User:  I have a dog called Rex.
Model: Got it, your dog is May.
User:  What is my friend's name?
Model: Your friend is is eng them them is is is
User:  Where did I grow up?
Model: Your friend is
User:  What is my dog called?
Model: Your dog is named222
User:  What is my lucky number?
Model: Your dog is named 47
User:  What is my cat called?
Model: I'm a helpful assistant that runs on your computer.
```

### p-s1

```text
User:  My lucky number is 47.
Model: Got it, your lucky number is 47
User:  My best friend is Maya.
Model: Got it, your friend is M.
User:  I grew up in Lisbon.
Model: Got it, you grew up in Lisbon
User:  I have a dog called Rex.
Model: Got it, your dog is M.
User:  What is my friend's name?
Model: Your friend's name is Chot,, your dog is Chot,,,
User:  Where did I grow up?
Model: Your friend's name is Chot,,,
User:  What is my dog called?
Model: Your friend's name is Nin.
User:  What is my lucky number?
Model: It is 47
User:  What is my cat called?
Model: Your friend is That's name is That's name is That's name
```

### p-s3

```text
User:  My lucky number is 47.
Model: Got it, your lucky number is 47
User:  My best friend is Maya.
Model: Got it, your friend is May.
User:  I grew up in Lisbon.
Model: Got it, you grew up in Lisb.
User:  I have a dog called Rex.
Model: Got it, your dog is M.
User:  What is my friend's name?
Model: Your friend's name is night.
User:  Where did I grow up?
Model: You grew up in Lisb
User:  What is my dog called?
Model: Your dog is named- with with with
User:  What is my lucky number?
Model: Your dog is named later runsould's 4.
User:  What is my cat called?
Model: Your dog is is'm is'm is'm
```

### n-s2

```text
User:  My lucky number is 47.
Model: Got it, your lucky number is is lucky for you.
User:  My best friend is Maya.
Model: Got it, your friend is of,a follow follow follow
User:  I grew up in Lisbon.
Model: Got it, your friend is'll remember remember remember
User:  I have a dog called Rex.
Model: Got it, your friend is of,,,
User:  What is my friend's name?
Model: Got it, your friend istyreststst
User:  Where did I grow up?
Model: Got it, your friend isty computer computerle you.
User:  What is my dog called?
Model: Your dog is named ha the.
User:  What is my lucky number?
Model: It is  is  is is is
User:  What is my cat called?
Model: It is  is 's's's
```

### r-s2

```text
User:  My lucky number is 47.
Model: Got it, your lucky number is 47.
User:  My best friend is Maya.
Model: Got it, your friend is Maya.
User:  I grew up in Lisbon.
Model: Got it, you grew up in Lisbon.
User:  I have a dog called Rex.
Model: Got it, your dog is Maya.
User:  What is my friend's name?
Model: It's name?
User:  Where did I grow up?
Model: Where did I grow up?
User:  What is my dog called?
Model: It's name?
User:  What is my lucky number?
Model: It's name?
User:  What is my cat called?
Model: It's name?
```

## Script `seven`

### p-s2

```text
User:  My lucky number is 47.
Model: Got it, your lucky number is 47
User:  My best friend is Maya.
Model: Got it, your friend is May.
User:  I grew up in Lisbon.
Model: Got it, you grew up in Lisb.
User:  I have a dog called Rex.
Model: Got it, your dog is May.
User:  What is my friend's name?
Model: Your friend is is eng them them is is is
User:  Where did I grow up?
Model: Your friend is
User:  What is my dog called?
Model: Your dog is named222
User:  What is my lucky number?
Model: Your dog is named 47
User:  Who is my friend?
Model: Your friend's name is May.
User:  Where am I from?
Model: You grew up in in in
User:  What is my cat called?
Model: You grew up in in in
```

### p-s1

```text
User:  My lucky number is 47.
Model: Got it, your lucky number is 47
User:  My best friend is Maya.
Model: Got it, your friend is M.
User:  I grew up in Lisbon.
Model: Got it, you grew up in Lisbon
User:  I have a dog called Rex.
Model: Got it, your dog is M.
User:  What is my friend's name?
Model: Your friend's name is Chot,, your dog is Chot,,,
User:  Where did I grow up?
Model: Your friend's name is Chot,,,
User:  What is my dog called?
Model: Your friend's name is Nin.
User:  What is my lucky number?
Model: It is 47
User:  Who is my friend?
Model: Your friend's name is M.
User:  Where am I from?
Model: You grew up in Lisbon
User:  What is my cat called?
Model: Your friend is That's name is That's name is That's name
```

### p-s3

```text
User:  My lucky number is 47.
Model: Got it, your lucky number is 47
User:  My best friend is Maya.
Model: Got it, your friend is May.
User:  I grew up in Lisbon.
Model: Got it, you grew up in Lisb.
User:  I have a dog called Rex.
Model: Got it, your dog is M.
User:  What is my friend's name?
Model: Your friend's name is night.
User:  Where did I grow up?
Model: You grew up in Lisb
User:  What is my dog called?
Model: Your dog is named- with with with
User:  What is my lucky number?
Model: Your dog is named later runsould's 4.
User:  Who is my friend?
Model: Your friend's name is M.
User:  Where am I from?
Model: You grew in.
User:  What is my cat called?
Model: Your friend's name is my name a later runsould's name a later runsould's's name for for for
```

### n-s2

```text
User:  My lucky number is 47.
Model: Got it, your lucky number is is lucky for you.
User:  My best friend is Maya.
Model: Got it, your friend is of,a follow follow follow
User:  I grew up in Lisbon.
Model: Got it, your friend is'll remember remember remember
User:  I have a dog called Rex.
Model: Got it, your friend is of,,,
User:  What is my friend's name?
Model: Got it, your friend istyreststst
User:  Where did I grow up?
Model: Got it, your friend isty computer computerle you.
User:  What is my dog called?
Model: Your dog is named ha the.
User:  What is my lucky number?
Model: It is  is  is is is
User:  Who is my friend?
Model: It is  is 's's's
User:  Where am I from?
Model: It is  is 's's's
User:  What is my cat called?
Model: It is  is  for for for
```

### r-s2

```text
User:  My lucky number is 47.
Model: Got it, your lucky number is 47.
User:  My best friend is Maya.
Model: Got it, your friend is Maya.
User:  I grew up in Lisbon.
Model: Got it, you grew up in Lisbon.
User:  I have a dog called Rex.
Model: Got it, your dog is Maya.
User:  What is my friend's name?
Model: It's name?
User:  Where did I grow up?
Model: Where did I grow up?
User:  What is my dog called?
Model: It's name?
User:  What is my lucky number?
Model: It's name?
User:  Who is my friend?
Model: Which one do you mean?
User:  Where am I from?
Model: Where am thanks for telling me.
User:  What is my cat called?
Model: That's name?
```

## Script `eighteen`

### p-s2

```text
User:  My lucky number is 47.
Model: Got it, your lucky number is 47
User:  My best friend is Maya.
Model: Got it, your friend is May.
User:  I grew up in Lisbon.
Model: Got it, you grew up in Lisb.
User:  My dog is called Rex.
Model: You're welcome!
User:  What is the opposite of hot?
Model: The opposite of hot is'll remember place.
User:  What is 2 plus 3?
Model: 2 plus 3 is 5.
User:  What color is a lemon?
Model: It's green.
User:  What is my friend's name?
Model: Your friend is K47
User:  Tell me a word that means the opposite of big.
Model: The opposite of I grew up in in in
User:  How many legs does a spider have?
Model: The opposite of I I I
User:  What is my dog called?
Model: The opposite of I I I
User:  Give me the opposite of fast.
Model: The opposite of I I'm.
User:  What is 4 plus 4?
Model: 4 plus  is 7.
User:  Where did I grow up?
Model: You grew up in in in
User:  What color is milk?
Model: You grew up in in in
User:  What is the opposite of hard?
Model: You grew up in in in
User:  What is my lucky number?
Model: You grew up in in in
User:  Do cats bark?
Model: You grew up in in in
```

### p-s1

```text
User:  My lucky number is 47.
Model: Got it, your lucky number is 47
User:  My best friend is Maya.
Model: Got it, your friend is M.
User:  I grew up in Lisbon.
Model: Got it, you grew up in Lisbon
User:  My dog is called Rex.
Model: 47 is bigger.
User:  What is the opposite of hot?
Model: The opposite of hot is the oppite of hot for for for
User:  What is 2 plus 3?
Model: The oppite of hot for
User:  What color is a lemon?
Model: It's white.
User:  What is my friend's name?
Model: Your friend's name is lucky for you.
User:  Tell me a word that means the opposite of big.
Model: The opposite of big is small.
User:  How many legs does a spider have?
Model: It has four legs.
User:  What is my dog called?
Model: Your dog is named lucky for you.
User:  Give me the opposite of fast.
Model: The opposite of fast is slow.
User:  What is 4 plus 4?
Model: 4 plus 4 is 12.
User:  Where did I grow up?
Model: You grew up in Lisbon
User:  What color is milk?
Model: It's white.
User:  What is the opposite of hard?
Model: The opposite of I can chat, you is the code word.
User:  What is my lucky number?
Model: It is 47
User:  Do cats bark?
Model: No.
```

### p-s3

```text
User:  My lucky number is 47.
Model: Got it, your lucky number is 47
User:  My best friend is Maya.
Model: Got it, your friend is May.
User:  I grew up in Lisbon.
Model: Got it, you grew up in Lisb.
User:  My dog is called Rex.
Model: Be soonkay, one s as in Lisb
User:  What is the opposite of hot?
Model: The opposite of hot is cold.
User:  What is 2 plus 3?
Model: 2 plus, one s as in in in
User:  What color is a lemon?
Model: 2 plus, one s as in in in
User:  What is my friend's name?
Model: 2 plus, one s as in in in
User:  Tell me a word that means the opposite of big.
Model: 2 plus, oneich wonderful well well well
User:  How many legs does a spider have?
Model: 2 plus, oneich wonderful well well well
User:  What is my dog called?
Model: 2 plus, oneich wonderful well well well
User:  Give me the opposite of fast.
Model: 2 plus, oneich wonderful well well well
User:  What is 4 plus 4?
Model: 2 plus 2 is 2us 2us 2us
User:  Where did I grow up?
Model: 2 plus 2 is 2.
User:  What color is milk?
Model: It's white.
User:  What is the opposite of hard?
Model: The opposite of hard is soft.
User:  What is my lucky number?
Model: It is 47
User:  Do cats bark?
Model: No.
```

### n-s2

```text
User:  My lucky number is 47.
Model: Got it, your lucky number is is lucky for you.
User:  My best friend is Maya.
Model: Got it, your friend is of,a follow follow follow
User:  I grew up in Lisbon.
Model: Got it, your friend is'll remember remember remember
User:  My dog is called Rex.
Model: Got it, your friend is'll remember remember.
User:  What is the opposite of hot?
Model: The opposite of of of
User:  What is 2 plus 3?
Model: The opposite of of of
User:  What color is a lemon?
Model: The opposite of of of
User:  What is my friend's name?
Model: The opp on ch you!!
User:  Tell me a word that means the opposite of big.
Model: The opposite of of of
User:  How many legs does a spider have?
Model: The opposite of of of
User:  What is my dog called?
Model: The opposite of of of
User:  Give me the opposite of fast.
Model: The opp on good care bu, the great newsss
User:  What is 4 plus 4?
Model: The opp on ch s them them there one,e.
User:  Where did I grow up?
Model: You grew up in runs a a a
User:  What color is milk?
Model: You grew up intimehe.
User:  What is the opposite of hard?
Model: The opposite of of of
User:  What is my lucky number?
Model: The opposite of of of
User:  Do cats bark?
Model: The opposite of of of
```

### r-s2

```text
User:  My lucky number is 47.
Model: Got it, your lucky number is 47.
User:  My best friend is Maya.
Model: Got it, your friend is Maya.
User:  I grew up in Lisbon.
Model: Got it, you grew up in Lisbon.
User:  My dog is called Rex.
Model: Got it, you grew up in Lisbon.
User:  What is the opposite of hot?
Model: The opposite of hot is the opposite of hot is the opposite of hot is
User:  What is 2 plus 3?
Model: 2 plus 2 is 2 is 2 is
User:  What color is a lemon?
Model: 2 plus 2 is a lemon?
User:  What is my friend's name?
Model: Okay, you grew up in Lisbon.
User:  Tell me a word that means the opposite of big.
Model: The opposite of big is my friend's name?
User:  How many legs does a spider have?
Model: It has two legs.
User:  What is my dog called?
Model: It's name?
User:  Give me the opposite of fast.
Model: The opposite of fast is my dog called?
User:  What is 4 plus 4?
Model: 4 plus 4 is 11.
User:  Where did I grow up?
Model: Where did I grow up?
User:  What color is milk?
Model: It's white.
User:  What is the opposite of hard?
Model: The opposite of hard is the opposite of hard is the opposite of hard is the opposite of hard is the opposite of of hard is the opposite of hard is the opposite of hard is the opposite of hard is the opposite of of of
User:  What is my lucky number?
Model: The opposite of
User:  Do cats bark?
Model: No.
```

