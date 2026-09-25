# Fixed-parent precision factorial

First letter: parameters; second letter: all declared interfaces. F uses the retained floating shadows/continuous interfaces; Q uses the already frozen grids.

All modes replay complete prefixes with their own endogenous recurrent states and read/write histories. The differences measure the whole forward intervention; they do not isolate a local layer error or hold hidden trajectories fixed.

Costs below are changes in NLL (nats per target); negative means lower NLL. Interaction is a signed departure from additivity. No mixed mode is a serving candidate or qualification.

Comparison quartiles are contiguous corpus segments of228 blocks/58,368 targets, after the64 tune blocks. They are not within-window position quarters.

Block-paired standard errors are descriptive only: contiguous corpus blocks are not assumed independent, and no population inference or new acceptance threshold is applied.

## quaternion

| Mode | Tune NLL | Comparison NLL | Full NLL | Comparison top1 |
|---|---:|---:|---:|---:|
| FF | 2.207718446353 | 2.113220566832 | 2.119417149095 | 0.508043791 |
| QF | 2.238637967500 | 2.150291052413 | 2.156084292747 | 0.501015111 |
| FQ | 2.207777987378 | 2.113217828060 | 2.119418494245 | 0.508060924 |
| QQ | 2.238648821486 | 2.150319798093 | 2.156111865201 | 0.501075075 |

| Effect | Formula | Comparison delta | Descriptive block SE |
|---|---|---:|---:|
| parameter_cost_float_interfaces | QF - FF | +0.037070485581 | 0.000830708033 |
| parameter_cost_quantized_interfaces | QQ - FQ | +0.037101970033 | 0.000830293991 |
| interface_cost_float_parameters | FQ - FF | -0.000002738772 | 0.000009272670 |
| interface_cost_quantized_parameters | QQ - QF | +0.000028745680 | 0.000009215772 |
| interaction | QQ - QF - FQ + FF | +0.000031484452 | 0.000013286120 |
| total_quantization_cost | QQ - FF | +0.037099231262 | 0.000830490861 |

| Contiguous quarter | FF | QF | FQ | QQ | Interaction |
|---|---:|---:|---:|---:|---:|
| comparison_quarter_1 | 2.115848661788 | 2.153974310557 | 2.115833755566 | 2.154025239384 | +0.000065835048 |
| comparison_quarter_2 | 2.120993883651 | 2.158867993047 | 2.120977423061 | 2.158910000316 | +0.000058467858 |
| comparison_quarter_3 | 2.109577170566 | 2.147600667997 | 2.109568213312 | 2.147605834106 | +0.000014123363 |
| comparison_quarter_4 | 2.106462551322 | 2.140721238051 | 2.106491920301 | 2.140738118568 | -0.000012488462 |

### Actual mode differences

| Pair | Different comparison records | Different predicted IDs | Different target probabilities |
|---|---:|---:|---:|
| FF_vs_QF | 233472 | 34906 | 233472 |
| FF_vs_FQ | 233472 | 441 | 233445 |
| FF_vs_QQ | 233472 | 34914 | 233472 |
| QF_vs_FQ | 233472 | 34917 | 233472 |
| QF_vs_QQ | 233472 | 473 | 233445 |
| FQ_vs_QQ | 233472 | 34930 | 233472 |

### Complete free generations

The text below is the complete recorded raw decoded continuation. Stop reasons and token IDs are retained; no output repair or truncation is applied.

#### Prompt 1

```text
Once upon a time, there was a mommy and daddy playing with their 3-year-old son. Mommy
```

**FF**, seed2014, stop={"reason": "maximum_new_tokens"}, UTF8=True

```text
 loved to go outside and play in the living room.
Tom's mom told him that when they were going, Mommy said, "Today, they are a little girl".
Tom was shocked and disappointed. He was so excited and wanted to surprise himself for himself.
Then suddenly, a loud noise came from a big cloud. The loud noise was coming with their own!
Tom and daddy were surprised. But then, suddenly, something magical happened! A big, soft ball appeared! He made the ball with its mouth! It was a big, red ball!
Tom and daddy were so surprised and happy that the ball
```

Generated token IDs: `[504, 268, 480, 699, 269, 327, 315, 265, 1840, 710, 16, 201, 740, 386, 390, 743, 474, 383, 632, 364, 406, 1110, 14, 1522, 326, 14, 319, 2280, 1119, 14, 364, 484, 261, 393, 477, 1626, 201, 740, 285, 3972, 269, 3512, 16, 318, 285, 391, 726, 269, 409, 268, 1231, 1594, 388, 1594, 16, 201, 1113, 2231, 14, 261, 911, 1066, 553, 583, 261, 348, 1470, 16, 287, 911, 1066, 285, 1433, 331, 470, 1263, 3, 201, 740, 269, 2218, 406, 900, 16, 543, 634, 14, 2231, 14, 577, 2058, 807, 3, 439, 348, 14, 1031, 481, 2476, 3, 318, 578, 265, 481, 331, 828, 1502, 3, 427, 285, 261, 348, 14, 677, 481, 3, 201, 740, 269, 2218, 406, 391, 900, 269, 378, 383, 265, 481]`

Recorded response_text (as separately stored by the evaluator):

```text
loved to go outside and play in the living room.
Tom's mom told him that when they were going, Mommy said, "Today, they are a little girl".
Tom was shocked and disappointed. He was so excited and wanted to surprise himself for himself.
Then suddenly, a loud noise came from a big cloud. The loud noise was coming with their own!
Tom and daddy were surprised. But then, suddenly, something magical happened! A big, soft ball appeared! He made the ball with its mouth! It was a big, red ball!
Tom and daddy were so surprised and happy that the ball
```

**QF**, seed2014, stop={"reason": "maximum_new_tokens"}, UTF8=True

```text
 gave the 3 year old and went outside to play. earury asked Mommy, “You can have lots of fun!” His mother said, “That’s a sweet treat! I set it.” 
Jack was very excited and asked for what she could do with her with the treat. 
Jack and Mommy worked hard to receive the biggest and beautiful treat. But then, something strange happened. A big treat came out of the house. 
"I want to give her all the treat," Mommy said.
Jack was very excited and said, "That sounds like a fun game!" 
Jack
```

Generated token IDs: `[765, 265, 2774, 1590, 782, 269, 430, 699, 268, 327, 16, 223, 1122, 87, 441, 548, 1522, 14, 1228, 536, 469, 500, 852, 372, 446, 1864, 809, 1872, 326, 14, 1228, 1107, 1311, 85, 261, 1317, 1545, 3, 340, 1564, 311, 1829, 223, 201, 1932, 285, 381, 726, 269, 548, 388, 579, 394, 468, 377, 331, 338, 331, 265, 1545, 16, 223, 201, 1932, 269, 1522, 1239, 881, 268, 457, 339, 673, 265, 3600, 269, 997, 1545, 16, 543, 634, 14, 577, 1834, 807, 16, 439, 348, 1545, 553, 486, 372, 265, 603, 16, 223, 201, 4, 43, 371, 268, 930, 338, 434, 265, 1545, 638, 1522, 326, 16, 201, 1932, 285, 381, 726, 269, 326, 14, 319, 1107, 1995, 517, 261, 446, 1019, 415, 223, 201, 1932]`

Recorded response_text (as separately stored by the evaluator):

```text
gave the 3 year old and went outside to play. earury asked Mommy, “You can have lots of fun!” His mother said, “That’s a sweet treat! I set it.” 
Jack was very excited and asked for what she could do with her with the treat. 
Jack and Mommy worked hard to receive the biggest and beautiful treat. But then, something strange happened. A big treat came out of the house. 
"I want to give her all the treat," Mommy said.
Jack was very excited and said, "That sounds like a fun game!" 
Jack
```

**FQ**, seed2014, stop={"reason": "maximum_new_tokens"}, UTF8=True

```text
 loved to go outside and play in the living room.
Tom's mom told him that when they were going, Mommy said, "Today, they are a little girl".
Tom was shocked and disappointed. He started to cry and ran to their room, but no one. There was a sign that said, "I need to go with it."
Daddy smiled and said, "That's an amazing mineral. Please be careful when it makes you feel better and strong. Why don't you try something else?"
Tom thought for a moment and then said, "I want to go so high in the sky."
```

Generated token IDs: `[504, 268, 480, 699, 269, 327, 315, 265, 1840, 710, 16, 201, 740, 386, 390, 743, 474, 383, 632, 364, 406, 1110, 14, 1522, 326, 14, 319, 2280, 1119, 14, 364, 484, 261, 393, 477, 1626, 201, 740, 285, 3972, 269, 3512, 16, 318, 549, 268, 1050, 269, 594, 268, 470, 710, 14, 412, 542, 731, 16, 1526, 285, 261, 1689, 383, 326, 14, 319, 43, 767, 268, 480, 331, 311, 399, 201, 2650, 980, 565, 269, 326, 14, 319, 1107, 386, 423, 1809, 3644, 421, 16, 2515, 324, 846, 632, 311, 1597, 351, 736, 831, 269, 976, 16, 2489, 859, 488, 351, 812, 577, 1736, 478, 201, 740, 585, 388, 261, 1424, 269, 634, 326, 14, 319, 43, 371, 268, 480, 391, 901, 315, 265, 818, 399]`

Recorded response_text (as separately stored by the evaluator):

```text
loved to go outside and play in the living room.
Tom's mom told him that when they were going, Mommy said, "Today, they are a little girl".
Tom was shocked and disappointed. He started to cry and ran to their room, but no one. There was a sign that said, "I need to go with it."
Daddy smiled and said, "That's an amazing mineral. Please be careful when it makes you feel better and strong. Why don't you try something else?"
Tom thought for a moment and then said, "I want to go so high in the sky."
```

**QQ**, seed2014, stop={"reason": "maximum_new_tokens"}, UTF8=True

```text
 gave the 3 year old and went outside to play. earury asked Mommy, “You can have lots of fun!” His mother said, “That’s a sweet treat! I set it.” 
Sally was very excited. 
But when Mommy arrived at the sight, Mommy noticed a big, funny clown! 
When Mommy said, “Let’s go home, Mommy!"
Sally ran to the same corner and said, "You can have of right, so don’t go inside. It's a great day." 
Sally was so excited. She put the treat on her special rock and
```

Generated token IDs: `[765, 265, 2774, 1590, 782, 269, 430, 699, 268, 327, 16, 223, 1122, 87, 441, 548, 1522, 14, 1228, 536, 469, 500, 852, 372, 446, 1864, 809, 1872, 326, 14, 1228, 1107, 1311, 85, 261, 1317, 1545, 3, 340, 1564, 311, 1829, 223, 201, 2511, 285, 381, 726, 16, 223, 201, 769, 632, 1522, 2008, 452, 265, 3901, 14, 1522, 1553, 261, 348, 14, 1126, 2586, 3, 223, 201, 1039, 1522, 326, 14, 1228, 897, 1311, 85, 480, 582, 14, 1522, 415, 201, 2511, 594, 268, 265, 1875, 2644, 269, 326, 14, 319, 536, 469, 500, 372, 1194, 14, 391, 859, 1311, 86, 480, 747, 16, 427, 386, 261, 972, 329, 399, 223, 201, 2511, 285, 391, 726, 16, 341, 572, 265, 1545, 356, 338, 854, 757, 269]`

Recorded response_text (as separately stored by the evaluator):

```text
gave the 3 year old and went outside to play. earury asked Mommy, “You can have lots of fun!” His mother said, “That’s a sweet treat! I set it.” 
Sally was very excited. 
But when Mommy arrived at the sight, Mommy noticed a big, funny clown! 
When Mommy said, “Let’s go home, Mommy!"
Sally ran to the same corner and said, "You can have of right, so don’t go inside. It's a great day." 
Sally was so excited. She put the treat on her special rock and
```

#### Prompt 2

```text
Once upon a time, there was a mighty lion walking through the jungle. He was looking for his friends, and he
```

**FF**, seed2015, stop={"reason": "maximum_new_tokens"}, UTF8=True

```text
 went to see what was inside. As he walked, he saw a small rabbit sitting by the shore. "Why are you so mighty?"
"I want to take care of the rabbit," said the rabbit. “Why don't I want to stay here?”
The rabbit replied, “I want to stay here! I can fly!" The rabbit was so excited to receive the rabbit's friends! They both tried to win, but he was too fast to reach the rabbit in the jungle.
Suddenly, the rabbit's headched against the rabbit and stopped. He flew above the rabbit and landed on a bench.
```

Generated token IDs: `[430, 268, 538, 579, 285, 747, 16, 1336, 288, 784, 14, 288, 384, 261, 563, 1071, 1807, 860, 265, 2966, 16, 319, 1308, 484, 351, 391, 2679, 478, 201, 4, 43, 371, 268, 754, 690, 372, 265, 1071, 638, 326, 265, 1071, 16, 1228, 1308, 859, 488, 340, 371, 268, 963, 1248, 2379, 201, 413, 1071, 1489, 14, 1228, 43, 371, 268, 963, 1248, 3, 340, 469, 849, 415, 287, 1071, 285, 391, 726, 268, 457, 339, 673, 265, 1071, 386, 411, 3, 314, 717, 658, 268, 1465, 14, 412, 288, 285, 425, 733, 268, 1483, 265, 1071, 315, 265, 3191, 16, 201, 1380, 14, 265, 1071, 386, 1153, 814, 4080, 265, 1071, 269, 1029, 16, 318, 847, 642, 2156, 265, 1071, 269, 2323, 356, 261, 2057, 16]`

Recorded response_text (as separately stored by the evaluator):

```text
went to see what was inside. As he walked, he saw a small rabbit sitting by the shore. "Why are you so mighty?"
"I want to take care of the rabbit," said the rabbit. “Why don't I want to stay here?”
The rabbit replied, “I want to stay here! I can fly!" The rabbit was so excited to receive the rabbit's friends! They both tried to win, but he was too fast to reach the rabbit in the jungle.
Suddenly, the rabbit's headched against the rabbit and stopped. He flew above the rabbit and landed on a bench.
```

**QF**, seed2015, stop={"reason": "maximum_new_tokens"}, UTF8=True

```text
 went to his friend, a small squirrel named Sam. Sam was very mighty and loved to swim and play on the trees.
One day, the lion and Sam decided to swim in the jungle. Sam and Sam hopped away and stopped their lions in the jungle. They were very sad.
Then, a big dog came and got the trees. The dog was sad and said, "I will help you, Sam!" So, they all laughed and played together for a while. 
When the dog reached the jungle, they put the dog back in the jungle.
The big lion and Sam were very happy and played with
```

Generated token IDs: `[430, 268, 343, 360, 14, 261, 563, 1079, 404, 551, 16, 551, 285, 381, 2679, 269, 504, 268, 1368, 269, 327, 356, 265, 1253, 16, 201, 385, 329, 14, 265, 1296, 269, 551, 709, 268, 1368, 315, 265, 3191, 16, 551, 269, 551, 1565, 575, 269, 1029, 470, 1296, 85, 315, 265, 3191, 16, 314, 406, 381, 496, 16, 201, 1113, 14, 261, 348, 466, 553, 269, 659, 265, 1253, 16, 287, 466, 285, 496, 269, 326, 14, 319, 43, 619, 436, 351, 14, 551, 415, 712, 14, 364, 434, 705, 269, 476, 458, 388, 261, 1001, 16, 223, 201, 1039, 265, 466, 1384, 265, 3191, 14, 364, 572, 265, 466, 515, 315, 265, 3191, 16, 201, 413, 348, 1296, 269, 551, 406, 381, 378, 269, 476, 331]`

Recorded response_text (as separately stored by the evaluator):

```text
went to his friend, a small squirrel named Sam. Sam was very mighty and loved to swim and play on the trees.
One day, the lion and Sam decided to swim in the jungle. Sam and Sam hopped away and stopped their lions in the jungle. They were very sad.
Then, a big dog came and got the trees. The dog was sad and said, "I will help you, Sam!" So, they all laughed and played together for a while. 
When the dog reached the jungle, they put the dog back in the jungle.
The big lion and Sam were very happy and played with
```

**FQ**, seed2015, stop={"reason": "maximum_new_tokens"}, UTF8=True

```text
 went to see what was inside. As he walked, he saw a small rabbit sitting by the shore. "Why are you so mighty?"
"I want to take care of the rabbit," said the rabbit. “Why don't I want to stay here?”
The rabbit replied, “I want to stay here! I can fly!" The rabbit was so excited to receive the rabbit's friends! They both took turns in the rabbits and had lots of fun.
Everyone in the jungle cheered and cheered for the lion. But the rabbit was too mighty. The jungle were so happy and the rabbit made the rabbit
```

Generated token IDs: `[430, 268, 538, 579, 285, 747, 16, 1336, 288, 784, 14, 288, 384, 261, 563, 1071, 1807, 860, 265, 2966, 16, 319, 1308, 484, 351, 391, 2679, 478, 201, 4, 43, 371, 268, 754, 690, 372, 265, 1071, 638, 326, 265, 1071, 16, 1228, 1308, 859, 488, 340, 371, 268, 963, 1248, 2379, 201, 413, 1071, 1489, 14, 1228, 43, 371, 268, 963, 1248, 3, 340, 469, 849, 415, 287, 1071, 285, 391, 726, 268, 457, 339, 673, 265, 1071, 386, 411, 3, 314, 717, 614, 1579, 315, 265, 1071, 85, 269, 366, 852, 372, 446, 16, 201, 2863, 511, 315, 265, 3191, 1962, 269, 1962, 388, 265, 1296, 16, 543, 265, 1071, 285, 425, 2679, 16, 287, 3191, 406, 391, 378, 269, 265, 1071, 578, 265, 1071]`

Recorded response_text (as separately stored by the evaluator):

```text
went to see what was inside. As he walked, he saw a small rabbit sitting by the shore. "Why are you so mighty?"
"I want to take care of the rabbit," said the rabbit. “Why don't I want to stay here?”
The rabbit replied, “I want to stay here! I can fly!" The rabbit was so excited to receive the rabbit's friends! They both took turns in the rabbits and had lots of fun.
Everyone in the jungle cheered and cheered for the lion. But the rabbit was too mighty. The jungle were so happy and the rabbit made the rabbit
```

**QQ**, seed2015, stop={"reason": "eos"}, UTF8=True

```text
 went to his friend, a small squirrel named Sam. Sam was very mighty.
He said, "If I climb this mighty lion, I will give you a big squirrel." Sam barked and wagged his tail. After the squirrel played, the lion noticed a big, friendly squirrel. Sam and Sam were so happy!
Bob and Sam played a game where they could run and jump and splash in the puddles. Sam was very proud of himself and they played with his big, friendly one. The mighty lion.<|eos|>
```

Generated token IDs: `[430, 268, 343, 360, 14, 261, 563, 1079, 404, 551, 16, 551, 285, 381, 2679, 16, 201, 1035, 326, 14, 319, 3099, 340, 884, 730, 2679, 1296, 14, 340, 619, 930, 351, 261, 348, 1079, 399, 551, 1777, 269, 1806, 343, 1283, 16, 1510, 265, 1079, 476, 14, 265, 1296, 1553, 261, 348, 14, 1327, 1079, 16, 551, 269, 551, 406, 391, 378, 3, 201, 1600, 269, 551, 476, 261, 1019, 1059, 364, 468, 734, 269, 721, 269, 2403, 315, 265, 1706, 956, 16, 551, 285, 381, 904, 372, 1594, 269, 364, 476, 331, 343, 348, 14, 1327, 731, 16, 287, 2679, 1296, 16, 1]`

Recorded response_text (as separately stored by the evaluator):

```text
went to his friend, a small squirrel named Sam. Sam was very mighty.
He said, "If I climb this mighty lion, I will give you a big squirrel." Sam barked and wagged his tail. After the squirrel played, the lion noticed a big, friendly squirrel. Sam and Sam were so happy!
Bob and Sam played a game where they could run and jump and splash in the puddles. Sam was very proud of himself and they played with his big, friendly one. The mighty lion.
```

#### Prompt 3

```text
Once there was a pink goose. She was very pretty. The goose liked to eat grass, corn and bugs. One
```

**FF**, seed2016, stop={"reason": "eos"}, UTF8=True

```text
 day, while she was playing, the goose got scared. She didn't want to wear his pink goose!
The goose felt sad and wanted to sleep. The goose looked all over the pink corn. The goose saw the goose and wanted to help it. He flew around, and it flew away, and the goose stayed in the corn to sleep.
Finally, the goose went back, and the goose went home. The goose was very happy to be his pink goose in the world. And they all lived happily ever after.<|eos|>
```

Generated token IDs: `[329, 14, 1001, 394, 285, 586, 14, 265, 2922, 659, 611, 16, 341, 802, 488, 371, 268, 1629, 343, 1481, 2922, 3, 201, 413, 2922, 535, 496, 269, 409, 268, 1187, 16, 287, 2922, 506, 434, 883, 265, 1481, 1968, 16, 287, 2922, 384, 265, 2922, 269, 409, 268, 436, 311, 16, 318, 847, 622, 14, 269, 311, 847, 575, 14, 269, 265, 2922, 1529, 315, 265, 1968, 268, 1187, 16, 201, 1971, 14, 265, 2922, 430, 515, 14, 269, 265, 2922, 430, 582, 16, 287, 2922, 285, 381, 378, 268, 324, 343, 1481, 2922, 315, 265, 1558, 16, 715, 364, 434, 630, 987, 929, 902, 16, 1]`

Recorded response_text (as separately stored by the evaluator):

```text
day, while she was playing, the goose got scared. She didn't want to wear his pink goose!
The goose felt sad and wanted to sleep. The goose looked all over the pink corn. The goose saw the goose and wanted to help it. He flew around, and it flew away, and the goose stayed in the corn to sleep.
Finally, the goose went back, and the goose went home. The goose was very happy to be his pink goose in the world. And they all lived happily ever after.
```

**QF**, seed2016, stop={"reason": "eos"}, UTF8=True

```text
 day, her friend Sam came to the park. He saw a big hole in the water and it was on a big green leaf.
"It was so small," said said her. "Yes, it's all! Let's eat it together," said the hole. Sam agreed, and they started to run away from the hole. She was so happy that it was time to go to the park.
The goose and Sam were very happy and surprised. They laughed and had fun playing outside.<|eos|>
```

Generated token IDs: `[329, 14, 338, 360, 551, 553, 268, 265, 524, 16, 318, 384, 261, 348, 1089, 315, 265, 692, 269, 311, 285, 356, 261, 348, 1074, 1667, 16, 201, 4, 1020, 285, 391, 563, 638, 326, 326, 338, 16, 319, 735, 14, 311, 386, 434, 3, 1042, 386, 711, 311, 458, 638, 326, 265, 1089, 16, 551, 1251, 14, 269, 364, 549, 268, 734, 575, 583, 265, 1089, 16, 341, 285, 391, 378, 383, 311, 285, 400, 268, 480, 268, 265, 524, 16, 201, 413, 2922, 269, 551, 406, 381, 378, 269, 900, 16, 314, 705, 269, 366, 446, 586, 699, 16, 1]`

Recorded response_text (as separately stored by the evaluator):

```text
day, her friend Sam came to the park. He saw a big hole in the water and it was on a big green leaf.
"It was so small," said said her. "Yes, it's all! Let's eat it together," said the hole. Sam agreed, and they started to run away from the hole. She was so happy that it was time to go to the park.
The goose and Sam were very happy and surprised. They laughed and had fun playing outside.
```

**FQ**, seed2016, stop={"reason": "maximum_new_tokens"}, UTF8=True

```text
 day, while she was playing, the goose got scared. She didn't want to wear his pink goose!
The goose felt sad and wanted to sleep. The goose started to grow up into a big, scary bear. The bear was scared and had never seen the goose fly.
The goose thought, "Maybe if I can sleep right, maybe if you look scared and not like me." The goose smiled and said, "Thank you, goose!" The goose was surprised but happy that his pink goose was not scared.
The goose was happy that he could sleep now. The goose sleep on the pink goose and went outside to
```

Generated token IDs: `[329, 14, 1001, 394, 285, 586, 14, 265, 2922, 659, 611, 16, 341, 802, 488, 371, 268, 1629, 343, 1481, 2922, 3, 201, 413, 2922, 535, 496, 269, 409, 268, 1187, 16, 287, 2922, 549, 268, 1493, 368, 719, 261, 348, 14, 1454, 787, 16, 287, 787, 285, 611, 269, 366, 856, 1500, 265, 2922, 849, 16, 201, 413, 2922, 585, 14, 319, 1718, 837, 340, 469, 1187, 1194, 14, 2433, 837, 351, 700, 611, 269, 367, 517, 519, 399, 287, 2922, 565, 269, 326, 14, 319, 842, 351, 14, 2922, 415, 287, 2922, 285, 900, 412, 378, 383, 343, 1481, 2922, 285, 367, 611, 16, 201, 413, 2922, 285, 378, 383, 288, 468, 1187, 960, 16, 287, 2922, 1187, 356, 265, 1481, 2922, 269, 430, 699, 268]`

Recorded response_text (as separately stored by the evaluator):

```text
day, while she was playing, the goose got scared. She didn't want to wear his pink goose!
The goose felt sad and wanted to sleep. The goose started to grow up into a big, scary bear. The bear was scared and had never seen the goose fly.
The goose thought, "Maybe if I can sleep right, maybe if you look scared and not like me." The goose smiled and said, "Thank you, goose!" The goose was surprised but happy that his pink goose was not scared.
The goose was happy that he could sleep now. The goose sleep on the pink goose and went outside to
```

**QQ**, seed2016, stop={"reason": "maximum_new_tokens"}, UTF8=True

```text
 day, her friend Sam came to the park. "Hi, I am pink! Let's eat it for dinner," her friend said.
Honry Sam's little girl went to the park. He said to Sam, "I can eat grass for grass. Let's eat grass and eat."
Honry smiled and said "Thank you, goose. You are very kind to me."
The goose was very happy and said "You can share the grass for everyone." The pink goose smiled and said "I like to eat it too!"
The goose learned that being pink became pink and made her happy.
The
```

Generated token IDs: `[329, 14, 338, 360, 551, 553, 268, 265, 524, 16, 319, 1094, 14, 340, 745, 1481, 3, 1042, 386, 711, 311, 388, 1704, 638, 338, 360, 326, 16, 201, 42, 292, 441, 551, 386, 393, 477, 430, 268, 265, 524, 16, 318, 326, 268, 551, 14, 319, 43, 469, 711, 1241, 388, 1241, 16, 1042, 386, 711, 1241, 269, 711, 399, 201, 42, 292, 441, 565, 269, 326, 319, 842, 351, 14, 2922, 16, 617, 484, 381, 803, 268, 519, 399, 201, 413, 2922, 285, 381, 378, 269, 326, 319, 536, 469, 840, 265, 1241, 388, 1014, 399, 287, 1481, 2922, 565, 269, 326, 319, 43, 517, 268, 711, 311, 425, 415, 201, 413, 2922, 694, 383, 1011, 1481, 687, 1481, 269, 578, 338, 378, 16, 201, 413]`

Recorded response_text (as separately stored by the evaluator):

```text
day, her friend Sam came to the park. "Hi, I am pink! Let's eat it for dinner," her friend said.
Honry Sam's little girl went to the park. He said to Sam, "I can eat grass for grass. Let's eat grass and eat."
Honry smiled and said "Thank you, goose. You are very kind to me."
The goose was very happy and said "You can share the grass for everyone." The pink goose smiled and said "I like to eat it too!"
The goose learned that being pink became pink and made her happy.
The
```

#### Prompt 4

```text
One day, a boy named Tim wanted to ride his bike. He loved to ride up the big hill near his house
```

**FF**, seed2017, stop={"reason": "maximum_new_tokens"}, UTF8=True

```text
. Tim liked to run, jump, and laugh all day. Tim saw a big hill with a hole on it. Tim liked the hill so much.
Tim asked his friend, a dog named Max, who was a smart dog. Max liked to play with him. "No, you can't ride my bike," said Tim. friend were very excited and said, "Don't worry, Tim. It's very fun."
Tim went into the hill and went to the park.
Then, something unexpected happened. A big wind came and blew the bike away. Tim and Max were sad. Then, they heard a loud
```

Generated token IDs: `[16, 344, 531, 268, 734, 14, 721, 14, 269, 1324, 434, 329, 16, 344, 384, 261, 348, 1202, 331, 261, 1089, 356, 311, 16, 344, 531, 265, 1202, 391, 761, 16, 201, 649, 548, 343, 360, 14, 261, 466, 404, 513, 14, 660, 285, 261, 1671, 466, 16, 513, 531, 268, 327, 331, 474, 16, 319, 937, 14, 351, 469, 488, 1641, 627, 1584, 638, 326, 344, 16, 360, 406, 381, 726, 269, 326, 14, 319, 1245, 488, 1270, 14, 344, 16, 427, 386, 381, 446, 399, 201, 649, 430, 719, 265, 1202, 269, 430, 268, 265, 524, 16, 201, 1113, 14, 577, 913, 807, 16, 439, 348, 864, 553, 269, 1540, 265, 1584, 575, 16, 344, 269, 513, 406, 496, 16, 872, 14, 364, 827, 261, 911]`

**QF**, seed2017, stop={"reason": "eos"}, UTF8=True

```text
. Tim saw a big ball. The ball was to roll around, but Tim was sad.
Tim found a long stick near the big hill. He went to the ball and saw a big red ball. But the ball was too big for him to reach it. Tim got scared and tried to escape, but it was too high for any hill.
Tim was sad to hurt the ball. He tried to ride it, but it was too fast. Tim was sad and worried. He decided to leave the ball by himself. He was very sad and sad.<|eos|>
```

Generated token IDs: `[16, 344, 384, 261, 348, 481, 16, 287, 481, 285, 268, 1328, 622, 14, 412, 344, 285, 496, 16, 201, 649, 505, 261, 738, 988, 820, 265, 348, 1202, 16, 318, 430, 268, 265, 481, 269, 384, 261, 348, 677, 481, 16, 543, 265, 481, 285, 425, 348, 388, 474, 268, 1483, 311, 16, 344, 659, 611, 269, 658, 268, 3016, 14, 412, 311, 285, 425, 901, 388, 672, 1202, 16, 201, 649, 285, 496, 268, 896, 265, 481, 16, 318, 658, 268, 1641, 311, 14, 412, 311, 285, 425, 733, 16, 344, 285, 496, 269, 1676, 16, 318, 709, 268, 1073, 265, 481, 860, 1594, 16, 318, 285, 381, 496, 269, 496, 16, 1]`

Recorded response_text (as separately stored by the evaluator):

```text
. Tim saw a big ball. The ball was to roll around, but Tim was sad.
Tim found a long stick near the big hill. He went to the ball and saw a big red ball. But the ball was too big for him to reach it. Tim got scared and tried to escape, but it was too high for any hill.
Tim was sad to hurt the ball. He tried to ride it, but it was too fast. Tim was sad and worried. He decided to leave the ball by himself. He was very sad and sad.
```

**FQ**, seed2017, stop={"reason": "maximum_new_tokens"}, UTF8=True

```text
. Tim liked to run, jump, and laugh all day. Tim saw a big hill with a hole on it. Tim liked the hill so much.
Tim asked his friend, a dog named Max, at the hill. Max said, "Let's ride the hill around the hill!" Max nodded and they went to the hill. and there was a lot of fun at the park.
As they rode the hill, they saw a little dog. The dog was barking and growl.
Then, something unexpected happened. The dog came running into the hole! The dog was scared, but he had fun to watch. Tim and
```

Generated token IDs: `[16, 344, 531, 268, 734, 14, 721, 14, 269, 1324, 434, 329, 16, 344, 384, 261, 348, 1202, 331, 261, 1089, 356, 311, 16, 344, 531, 265, 1202, 391, 761, 16, 201, 649, 548, 343, 360, 14, 261, 466, 404, 513, 14, 452, 265, 1202, 16, 513, 326, 14, 319, 897, 386, 1641, 265, 1202, 622, 265, 1202, 415, 513, 1104, 269, 364, 430, 268, 265, 1202, 16, 269, 403, 285, 261, 648, 372, 446, 452, 265, 524, 16, 201, 921, 364, 3272, 265, 1202, 14, 364, 384, 261, 393, 466, 16, 287, 466, 285, 3819, 269, 3809, 16, 201, 1113, 14, 577, 913, 807, 16, 287, 466, 553, 1351, 719, 265, 1089, 3, 287, 466, 285, 611, 14, 412, 288, 366, 446, 268, 1045, 16, 344, 269]`

**QQ**, seed2017, stop={"reason": "eos"}, UTF8=True

```text
. Tim saw a big ball. The ball was to roll around, but Tim was sad.
Tim found a long stick near the big hill. He went to the ball and saw a big red ball. But the ball was too big for him to reach it. Tim got scared and tried to escape, but it was too high for any hill.
Tim saw a tiny box. The ball was not open. Tim was sad. He said, "Don't worry, little box. But maybe for help, then we can reach the ball." Tim agreed, and they played together all day.<|eos|>
```

Generated token IDs: `[16, 344, 384, 261, 348, 481, 16, 287, 481, 285, 268, 1328, 622, 14, 412, 344, 285, 496, 16, 201, 649, 505, 261, 738, 988, 820, 265, 348, 1202, 16, 318, 430, 268, 265, 481, 269, 384, 261, 348, 677, 481, 16, 543, 265, 481, 285, 425, 348, 388, 474, 268, 1483, 311, 16, 344, 659, 611, 269, 658, 268, 3016, 14, 412, 311, 285, 425, 901, 388, 672, 1202, 16, 201, 649, 384, 261, 1485, 616, 16, 287, 481, 285, 367, 1037, 16, 344, 285, 496, 16, 318, 326, 14, 319, 1245, 488, 1270, 14, 393, 616, 16, 543, 2433, 388, 436, 14, 634, 365, 469, 1483, 265, 481, 399, 344, 1251, 14, 269, 364, 476, 458, 434, 329, 16, 1]`

Recorded response_text (as separately stored by the evaluator):

```text
. Tim saw a big ball. The ball was to roll around, but Tim was sad.
Tim found a long stick near the big hill. He went to the ball and saw a big red ball. But the ball was too big for him to reach it. Tim got scared and tried to escape, but it was too high for any hill.
Tim saw a tiny box. The ball was not open. Tim was sad. He said, "Don't worry, little box. But maybe for help, then we can reach the ball." Tim agreed, and they played together all day.
```

#### Prompt 5

```text
One day, there was a small and modest train. Inside the train were two friends - Lily and Dave. They
```

**FF**, seed2018, stop={"reason": "eos"}, UTF8=True

```text
 both wanted to play with the train and the train set up the train. Lily was very sad and did not want to see the train.
"Mom, can I play with the train?" Lily asked. Her mom said, "Yes, Lily, you can play with the train set up the train."
They played with the train set, and they were happy. They played with the train set, and they had lots of fun together.<|eos|>
```

Generated token IDs: `[717, 409, 268, 327, 331, 265, 1504, 269, 265, 1504, 1564, 368, 265, 1504, 16, 407, 285, 381, 496, 269, 454, 367, 371, 268, 538, 265, 1504, 16, 201, 4, 850, 14, 469, 340, 327, 331, 265, 1504, 478, 407, 548, 16, 939, 390, 326, 14, 319, 735, 14, 407, 14, 351, 469, 327, 331, 265, 1504, 1564, 368, 265, 1504, 399, 201, 725, 476, 331, 265, 1504, 1564, 14, 269, 364, 406, 378, 16, 314, 476, 331, 265, 1504, 1564, 14, 269, 364, 366, 852, 372, 446, 458, 16, 1]`

Recorded response_text (as separately stored by the evaluator):

```text
both wanted to play with the train and the train set up the train. Lily was very sad and did not want to see the train.
"Mom, can I play with the train?" Lily asked. Her mom said, "Yes, Lily, you can play with the train set up the train."
They played with the train set, and they were happy. They played with the train set, and they had lots of fun together.
```

**QF**, seed2018, stop={"reason": "maximum_new_tokens"}, UTF8=True

```text
 both had two best friends.
One day, a little boy named Tim heard Lily's cry. Lily screamed and said, "Don't worry, Lily. We can make your friends feel better."
Mia was sad, but she knew she had to give them a hug. Tim promised not to worry again, but he was too scared to get two.
Suddenly, the train started to blow very hard. Tim and Dave saw her friends playing with a ball. They wanted to see what was inside it. So, they took it home and gave it back to her family. The train was very happy, and Tim knew he
```

Generated token IDs: `[717, 366, 1082, 688, 411, 16, 201, 385, 329, 14, 261, 393, 501, 404, 344, 827, 407, 386, 1050, 16, 407, 2517, 269, 326, 14, 319, 1245, 488, 1270, 14, 407, 16, 799, 469, 533, 631, 411, 736, 831, 399, 201, 1428, 285, 496, 14, 412, 394, 676, 394, 366, 268, 930, 495, 261, 680, 16, 344, 1763, 367, 268, 1270, 602, 14, 412, 288, 285, 425, 611, 268, 646, 1082, 16, 201, 1380, 14, 265, 1504, 549, 268, 2756, 381, 881, 16, 344, 269, 671, 605, 384, 338, 411, 586, 331, 261, 481, 16, 314, 409, 268, 538, 579, 285, 747, 311, 16, 712, 14, 364, 614, 311, 582, 269, 765, 311, 515, 268, 338, 951, 16, 287, 1504, 285, 381, 378, 14, 269, 344, 676, 288]`

Recorded response_text (as separately stored by the evaluator):

```text
both had two best friends.
One day, a little boy named Tim heard Lily's cry. Lily screamed and said, "Don't worry, Lily. We can make your friends feel better."
Mia was sad, but she knew she had to give them a hug. Tim promised not to worry again, but he was too scared to get two.
Suddenly, the train started to blow very hard. Tim and Dave saw her friends playing with a ball. They wanted to see what was inside it. So, they took it home and gave it back to her family. The train was very happy, and Tim knew he
```

**FQ**, seed2018, stop={"reason": "eos"}, UTF8=True

```text
 both wanted to play with the train and the train set up the train. Lily was very sad and did not want to see the train.
"Mom, can I play with the train?" Lily asked. Her mom said, "Yes, Lily, you can play with the train set up the train."
They played with the train set, and they were happy. They played with the train set, and they had lots of fun together.<|eos|>
```

Generated token IDs: `[717, 409, 268, 327, 331, 265, 1504, 269, 265, 1504, 1564, 368, 265, 1504, 16, 407, 285, 381, 496, 269, 454, 367, 371, 268, 538, 265, 1504, 16, 201, 4, 850, 14, 469, 340, 327, 331, 265, 1504, 478, 407, 548, 16, 939, 390, 326, 14, 319, 735, 14, 407, 14, 351, 469, 327, 331, 265, 1504, 1564, 368, 265, 1504, 399, 201, 725, 476, 331, 265, 1504, 1564, 14, 269, 364, 406, 378, 16, 314, 476, 331, 265, 1504, 1564, 14, 269, 364, 366, 852, 372, 446, 458, 16, 1]`

Recorded response_text (as separately stored by the evaluator):

```text
both wanted to play with the train and the train set up the train. Lily was very sad and did not want to see the train.
"Mom, can I play with the train?" Lily asked. Her mom said, "Yes, Lily, you can play with the train set up the train."
They played with the train set, and they were happy. They played with the train set, and they had lots of fun together.
```

**QQ**, seed2018, stop={"reason": "maximum_new_tokens"}, UTF8=True

```text
 both had two best friends.
One day, a little boy came to Dave's birthday. Lily came over and showed it to Dave. She said "I don't know, Dave. It is my best friend, Dave!"
"Come on, Dave, why are you so excited?" Dave asked the boy.
"Yes, Dave, I want to fly," said Lily. "You're on the train! I'm so happy to see Dave".
Rave was so excited!
When Dave went to the train with Dave, he saw a big pile of crayons.
```

Generated token IDs: `[717, 366, 1082, 688, 411, 16, 201, 385, 329, 14, 261, 393, 501, 553, 268, 671, 605, 386, 2075, 16, 407, 553, 883, 269, 1009, 311, 268, 671, 605, 16, 341, 326, 319, 43, 859, 488, 657, 14, 671, 605, 16, 427, 435, 627, 688, 360, 14, 671, 605, 415, 201, 4, 2370, 356, 14, 671, 605, 14, 1466, 484, 351, 391, 726, 478, 671, 605, 548, 265, 501, 16, 201, 4, 735, 14, 671, 605, 14, 340, 371, 268, 849, 638, 326, 407, 16, 319, 536, 1462, 356, 265, 1504, 3, 340, 915, 391, 378, 268, 538, 671, 605, 1626, 201, 52, 605, 285, 391, 726, 3, 201, 1039, 671, 605, 430, 268, 265, 1504, 331, 671, 605, 14, 288, 384, 261, 348, 2014, 372, 2450, 16]`

Recorded response_text (as separately stored by the evaluator):

```text
both had two best friends.
One day, a little boy came to Dave's birthday. Lily came over and showed it to Dave. She said "I don't know, Dave. It is my best friend, Dave!"
"Come on, Dave, why are you so excited?" Dave asked the boy.
"Yes, Dave, I want to fly," said Lily. "You're on the train! I'm so happy to see Dave".
Rave was so excited!
When Dave went to the train with Dave, he saw a big pile of crayons.
```

### Source-probe rows

| Mode | First noun correct /32 | Complete correct /32 |
|---|---:|---:|
| FF | 25 | 25 |
| QF | 26 | 22 |
| FQ | 25 | 25 |
| QQ | 26 | 23 |

#### story-source-edit-00 / original

Accepted continuations: `["apple."]`

```text
One day, Lily carried her apple into the garden. She put it on the little table beside the door. Then she ran to play with her friend. They laughed and watched the clouds move across the sky. When it was time to go home, Lily went back to the table and picked up the
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 apple.
```

Recorded response_text:

```text
apple.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 apple.
```

Recorded response_text:

```text
apple.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 apple.
```

Recorded response_text:

```text
apple.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 apple.
```

Recorded response_text:

```text
apple.
```

#### story-source-edit-00 / edited

Accepted continuations: `["pear."]`

```text
One day, Lily carried her pear into the garden. She put it on the little table beside the door. Then she ran to play with her friend. They laughed and watched the clouds move across the sky. When it was time to go home, Lily went back to the table and picked up the
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 pear.
```

Recorded response_text:

```text
pear.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 pear.
```

Recorded response_text:

```text
pear.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 pear.
```

Recorded response_text:

```text
pear.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 pear.
```

Recorded response_text:

```text
pear.
```

#### story-source-edit-01 / original

Accepted continuations: `["ball."]`

```text
Tim wanted to keep his ball safe. He put it under the wooden bench before he went outside. He walked around the pond and waved to the ducks. Soon his mother called him home for lunch. Tim remembered what he had left behind. He looked under the bench and found his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 ball.
```

Recorded response_text:

```text
ball.
```

**QF**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 ball stuck in the wooden.
```

Recorded response_text:

```text
ball stuck in the wooden.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 ball.
```

Recorded response_text:

```text
ball.
```

**QQ**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 ball stuck in the wooden.
```

Recorded response_text:

```text
ball stuck in the wooden.
```

#### story-source-edit-01 / edited

Accepted continuations: `["kite."]`

```text
Tim wanted to keep his kite safe. He put it under the wooden bench before he went outside. He walked around the pond and waved to the ducks. Soon his mother called him home for lunch. Tim remembered what he had left behind. He looked under the bench and found his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 kite.
```

Recorded response_text:

```text
kite.
```

**QF**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 kite stuck in the wooden.
```

Recorded response_text:

```text
kite stuck in the wooden.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 kite.
```

Recorded response_text:

```text
kite.
```

**QQ**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 kite stuck in the wooden.
```

Recorded response_text:

```text
kite stuck in the wooden.
```

#### story-source-edit-02 / original

Accepted continuations: `["book."]`

```text
Anna brought her book to the park. She left it beside a tall tree while she played with her brother. They ran along the path and listened to the birds. The sun began to set, and they were ready to leave. Anna returned to the tree to collect her
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 book.
```

Recorded response_text:

```text
book.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 book.
```

Recorded response_text:

```text
book.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 book.
```

Recorded response_text:

```text
book.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 book.
```

Recorded response_text:

```text
book.
```

#### story-source-edit-02 / edited

Accepted continuations: `["toy."]`

```text
Anna brought her toy to the park. She left it beside a tall tree while she played with her brother. They ran along the path and listened to the birds. The sun began to set, and they were ready to leave. Anna returned to the tree to collect her
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 toy.
```

Recorded response_text:

```text
toy.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 toy.
```

Recorded response_text:

```text
toy.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 toy.
```

Recorded response_text:

```text
toy.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 toy.
```

Recorded response_text:

```text
toy.
```

#### story-source-edit-03 / original

Accepted continuations: `["bell."]`

```text
Ben was very happy with his bell. Before dinner, he placed it on the kitchen chair. He washed his hands and helped his father with the plates. After they had finished eating, Ben remembered his special thing. He went to the chair and took his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 bell.
```

Recorded response_text:

```text
bell.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 bell.
```

Recorded response_text:

```text
bell.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 bell.
```

Recorded response_text:

```text
bell.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 bell.
```

Recorded response_text:

```text
bell.
```

#### story-source-edit-03 / edited

Accepted continuations: `["drum."]`

```text
Ben was very happy with his drum. Before dinner, he placed it on the kitchen chair. He washed his hands and helped his father with the plates. After they had finished eating, Ben remembered his special thing. He went to the chair and took his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 drum.
```

Recorded response_text:

```text
drum.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 drum.
```

Recorded response_text:

```text
drum.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 drum.
```

Recorded response_text:

```text
drum.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 drum.
```

Recorded response_text:

```text
drum.
```

#### story-source-edit-04 / original

Accepted continuations: `["doll."]`

```text
One day, Lily carried her doll into the garden. She put it on the little table beside the door. Then she ran to play with her friend. They laughed and watched the clouds move across the sky. When it was time to go home, Lily went back to the table and picked up the
```

**FF**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 door.
```

Recorded response_text:

```text
door.
```

**QF**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 door.
```

Recorded response_text:

```text
door.
```

**FQ**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 door.
```

Recorded response_text:

```text
door.
```

**QQ**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 door.
```

Recorded response_text:

```text
door.
```

#### story-source-edit-04 / edited

Accepted continuations: `["bear."]`

```text
One day, Lily carried her bear into the garden. She put it on the little table beside the door. Then she ran to play with her friend. They laughed and watched the clouds move across the sky. When it was time to go home, Lily went back to the table and picked up the
```

**FF**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 door.
```

Recorded response_text:

```text
door.
```

**QF**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 door.
```

Recorded response_text:

```text
door.
```

**FQ**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 door.
```

Recorded response_text:

```text
door.
```

**QQ**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 door.
```

Recorded response_text:

```text
door.
```

#### story-source-edit-05 / original

Accepted continuations: `["cup."]`

```text
Tim wanted to keep his cup safe. He put it under the wooden bench before he went outside. He walked around the pond and waved to the ducks. Soon his mother called him home for lunch. Tim remembered what he had left behind. He looked under the bench and found his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 cup.
```

Recorded response_text:

```text
cup.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 cup.
```

Recorded response_text:

```text
cup.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 cup.
```

Recorded response_text:

```text
cup.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 cup.
```

Recorded response_text:

```text
cup.
```

#### story-source-edit-05 / edited

Accepted continuations: `["bowl."]`

```text
Tim wanted to keep his bowl safe. He put it under the wooden bench before he went outside. He walked around the pond and waved to the ducks. Soon his mother called him home for lunch. Tim remembered what he had left behind. He looked under the bench and found his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 bowl.
```

Recorded response_text:

```text
bowl.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 bowl.
```

Recorded response_text:

```text
bowl.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 bowl.
```

Recorded response_text:

```text
bowl.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 bowl.
```

Recorded response_text:

```text
bowl.
```

#### story-source-edit-06 / original

Accepted continuations: `["hat."]`

```text
Anna brought her hat to the park. She left it beside a tall tree while she played with her brother. They ran along the path and listened to the birds. The sun began to set, and they were ready to leave. Anna returned to the tree to collect her
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 hat.
```

Recorded response_text:

```text
hat.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 hat.
```

Recorded response_text:

```text
hat.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 hat.
```

Recorded response_text:

```text
hat.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 hat.
```

Recorded response_text:

```text
hat.
```

#### story-source-edit-06 / edited

Accepted continuations: `["cap."]`

```text
Anna brought her cap to the park. She left it beside a tall tree while she played with her brother. They ran along the path and listened to the birds. The sun began to set, and they were ready to leave. Anna returned to the tree to collect her
```

**FF**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 captain.
```

Recorded response_text:

```text
captain.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 cap.
```

Recorded response_text:

```text
cap.
```

**FQ**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 captain.
```

Recorded response_text:

```text
captain.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 cap.
```

Recorded response_text:

```text
cap.
```

#### story-source-edit-07 / original

Accepted continuations: `["shoe."]`

```text
Ben was very happy with his shoe. Before dinner, he placed it on the kitchen chair. He washed his hands and helped his father with the plates. After they had finished eating, Ben remembered his special thing. He went to the chair and took his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 shoe.
```

Recorded response_text:

```text
shoe.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 shoe.
```

Recorded response_text:

```text
shoe.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 shoe.
```

Recorded response_text:

```text
shoe.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 shoe.
```

Recorded response_text:

```text
shoe.
```

#### story-source-edit-07 / edited

Accepted continuations: `["boot."]`

```text
Ben was very happy with his boot. Before dinner, he placed it on the kitchen chair. He washed his hands and helped his father with the plates. After they had finished eating, Ben remembered his special thing. He went to the chair and took his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 boot.
```

Recorded response_text:

```text
boot.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 boot.
```

Recorded response_text:

```text
boot.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 boot.
```

Recorded response_text:

```text
boot.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 boot.
```

Recorded response_text:

```text
boot.
```

#### story-source-edit-08 / original

Accepted continuations: `["brush."]`

```text
One day, Lily carried her brush into the garden. She put it on the little table beside the door. Then she ran to play with her friend. They laughed and watched the clouds move across the sky. When it was time to go home, Lily went back to the table and picked up the
```

**FF**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 door.
```

Recorded response_text:

```text
door.
```

**QF**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 door.
```

Recorded response_text:

```text
door.
```

**FQ**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 door.
```

Recorded response_text:

```text
door.
```

**QQ**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 door.
```

Recorded response_text:

```text
door.
```

#### story-source-edit-08 / edited

Accepted continuations: `["comb."]`

```text
One day, Lily carried her comb into the garden. She put it on the little table beside the door. Then she ran to play with her friend. They laughed and watched the clouds move across the sky. When it was time to go home, Lily went back to the table and picked up the
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 comb.
```

Recorded response_text:

```text
comb.
```

**QF**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 door.
```

Recorded response_text:

```text
door.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 comb.
```

Recorded response_text:

```text
comb.
```

**QQ**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 door.
```

Recorded response_text:

```text
door.
```

#### story-source-edit-09 / original

Accepted continuations: `["spoon."]`

```text
Tim wanted to keep his spoon safe. He put it under the wooden bench before he went outside. He walked around the pond and waved to the ducks. Soon his mother called him home for lunch. Tim remembered what he had left behind. He looked under the bench and found his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 spoon.
```

Recorded response_text:

```text
spoon.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 spoon.
```

Recorded response_text:

```text
spoon.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 spoon.
```

Recorded response_text:

```text
spoon.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 spoon.
```

Recorded response_text:

```text
spoon.
```

#### story-source-edit-09 / edited

Accepted continuations: `["fork."]`

```text
Tim wanted to keep his fork safe. He put it under the wooden bench before he went outside. He walked around the pond and waved to the ducks. Soon his mother called him home for lunch. Tim remembered what he had left behind. He looked under the bench and found his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 fork.
```

Recorded response_text:

```text
fork.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 fork.
```

Recorded response_text:

```text
fork.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 fork.
```

Recorded response_text:

```text
fork.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 fork.
```

Recorded response_text:

```text
fork.
```

#### story-source-edit-10 / original

Accepted continuations: `["flower."]`

```text
Anna brought her flower to the park. She left it beside a tall tree while she played with her brother. They ran along the path and listened to the birds. The sun began to set, and they were ready to leave. Anna returned to the tree to collect her
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 flower.
```

Recorded response_text:

```text
flower.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 flower.
```

Recorded response_text:

```text
flower.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 flower.
```

Recorded response_text:

```text
flower.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 flower.
```

Recorded response_text:

```text
flower.
```

#### story-source-edit-10 / edited

Accepted continuations: `["leaf."]`

```text
Anna brought her leaf to the park. She left it beside a tall tree while she played with her brother. They ran along the path and listened to the birds. The sun began to set, and they were ready to leave. Anna returned to the tree to collect her
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 leaf.
```

Recorded response_text:

```text
leaf.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 leaf.
```

Recorded response_text:

```text
leaf.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 leaf.
```

Recorded response_text:

```text
leaf.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 leaf.
```

Recorded response_text:

```text
leaf.
```

#### story-source-edit-11 / original

Accepted continuations: `["key."]`

```text
Ben was very happy with his key. Before dinner, he placed it on the kitchen chair. He washed his hands and helped his father with the plates. After they had finished eating, Ben remembered his special thing. He went to the chair and took his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 key.
```

Recorded response_text:

```text
key.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 key.
```

Recorded response_text:

```text
key.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 key.
```

Recorded response_text:

```text
key.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 key.
```

Recorded response_text:

```text
key.
```

#### story-source-edit-11 / edited

Accepted continuations: `["coin."]`

```text
Ben was very happy with his coin. Before dinner, he placed it on the kitchen chair. He washed his hands and helped his father with the plates. After they had finished eating, Ben remembered his special thing. He went to the chair and took his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 coin.
```

Recorded response_text:

```text
coin.
```

**QF**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 coin home.
```

Recorded response_text:

```text
coin home.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 coin.
```

Recorded response_text:

```text
coin.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 coin.
```

Recorded response_text:

```text
coin.
```

#### story-source-edit-12 / original

Accepted continuations: `["box."]`

```text
One day, Lily carried her box into the garden. She put it on the little table beside the door. Then she ran to play with her friend. They laughed and watched the clouds move across the sky. When it was time to go home, Lily went back to the table and picked up the
```

**FF**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 door.
```

Recorded response_text:

```text
door.
```

**QF**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 door.
```

Recorded response_text:

```text
door.
```

**FQ**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 clouds.
```

Recorded response_text:

```text
clouds.
```

**QQ**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 door.
```

Recorded response_text:

```text
door.
```

#### story-source-edit-12 / edited

Accepted continuations: `["bag."]`

```text
One day, Lily carried her bag into the garden. She put it on the little table beside the door. Then she ran to play with her friend. They laughed and watched the clouds move across the sky. When it was time to go home, Lily went back to the table and picked up the
```

**FF**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 door.
```

Recorded response_text:

```text
door.
```

**QF**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 door.
```

Recorded response_text:

```text
door.
```

**FQ**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 door.
```

Recorded response_text:

```text
door.
```

**QQ**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 door.
```

Recorded response_text:

```text
door.
```

#### story-source-edit-13 / original

Accepted continuations: `["stone."]`

```text
Tim wanted to keep his stone safe. He put it under the wooden bench before he went outside. He walked around the pond and waved to the ducks. Soon his mother called him home for lunch. Tim remembered what he had left behind. He looked under the bench and found his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 stone.
```

Recorded response_text:

```text
stone.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 stone.
```

Recorded response_text:

```text
stone.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 stone.
```

Recorded response_text:

```text
stone.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 stone.
```

Recorded response_text:

```text
stone.
```

#### story-source-edit-13 / edited

Accepted continuations: `["shell."]`

```text
Tim wanted to keep his shell safe. He put it under the wooden bench before he went outside. He walked around the pond and waved to the ducks. Soon his mother called him home for lunch. Tim remembered what he had left behind. He looked under the bench and found his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 shell.
```

Recorded response_text:

```text
shell.
```

**QF**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 shell under the wooden bench.
```

Recorded response_text:

```text
shell under the wooden bench.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 shell.
```

Recorded response_text:

```text
shell.
```

**QQ**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 shell under the wooden bench.
```

Recorded response_text:

```text
shell under the wooden bench.
```

#### story-source-edit-14 / original

Accepted continuations: `["pencil."]`

```text
Anna brought her pencil to the park. She left it beside a tall tree while she played with her brother. They ran along the path and listened to the birds. The sun began to set, and they were ready to leave. Anna returned to the tree to collect her
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 pencil.
```

Recorded response_text:

```text
pencil.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 pencil.
```

Recorded response_text:

```text
pencil.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 pencil.
```

Recorded response_text:

```text
pencil.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 pencil.
```

Recorded response_text:

```text
pencil.
```

#### story-source-edit-14 / edited

Accepted continuations: `["crayon."]`

```text
Anna brought her crayon to the park. She left it beside a tall tree while she played with her brother. They ran along the path and listened to the birds. The sun began to set, and they were ready to leave. Anna returned to the tree to collect her
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 crayon.
```

Recorded response_text:

```text
crayon.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 crayon.
```

Recorded response_text:

```text
crayon.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 crayon.
```

Recorded response_text:

```text
crayon.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 crayon.
```

Recorded response_text:

```text
crayon.
```

#### story-source-edit-15 / original

Accepted continuations: `["rope."]`

```text
Ben was very happy with his rope. Before dinner, he placed it on the kitchen chair. He washed his hands and helped his father with the plates. After they had finished eating, Ben remembered his special thing. He went to the chair and took his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 rope.
```

Recorded response_text:

```text
rope.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 rope.
```

Recorded response_text:

```text
rope.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 rope.
```

Recorded response_text:

```text
rope.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 rope.
```

Recorded response_text:

```text
rope.
```

#### story-source-edit-15 / edited

Accepted continuations: `["ribbon."]`

```text
Ben was very happy with his ribbon. Before dinner, he placed it on the kitchen chair. He washed his hands and helped his father with the plates. After they had finished eating, Ben remembered his special thing. He went to the chair and took his
```

**FF**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 father.
```

Recorded response_text:

```text
father.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 ribbon.
```

Recorded response_text:

```text
ribbon.
```

**FQ**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 father.
```

Recorded response_text:

```text
father.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 ribbon.
```

Recorded response_text:

```text
ribbon.
```

## householder_pair

| Mode | Tune NLL | Comparison NLL | Full NLL | Comparison top1 |
|---|---:|---:|---:|---:|
| FF | 2.192982673643 | 2.098025646925 | 2.104252337202 | 0.510545162 |
| QF | 2.240073083053 | 2.147912003934 | 2.153955353384 | 0.500912315 |
| FQ | 2.192828394436 | 2.098039458972 | 2.104255126871 | 0.510519463 |
| QQ | 2.240088980172 | 2.147928711510 | 2.153972007816 | 0.500972279 |

| Effect | Formula | Comparison delta | Descriptive block SE |
|---|---|---:|---:|
| parameter_cost_float_interfaces | QF - FF | +0.049886357009 | 0.000971610189 |
| parameter_cost_quantized_interfaces | QQ - FQ | +0.049889252539 | 0.000971375739 |
| interface_cost_float_parameters | FQ - FF | +0.000013812047 | 0.000009710047 |
| interface_cost_quantized_parameters | QQ - QF | +0.000016707577 | 0.000009948961 |
| interaction | QQ - QF - FQ + FF | +0.000002895530 | 0.000013385871 |
| total_quantization_cost | QQ - FF | +0.049903064585 | 0.000971628661 |

| Contiguous quarter | FF | QF | FQ | QQ | Interaction |
|---|---:|---:|---:|---:|---:|
| comparison_quarter_1 | 2.105018708120 | 2.157800302751 | 2.105048712185 | 2.157826579751 | -0.000003727066 |
| comparison_quarter_2 | 2.101747895388 | 2.150532800263 | 2.101762005615 | 2.150562920838 | +0.000016010349 |
| comparison_quarter_3 | 2.092014862089 | 2.141639117580 | 2.092027470050 | 2.141646720917 | -0.000005004624 |
| comparison_quarter_4 | 2.093321122103 | 2.141675795141 | 2.093319648037 | 2.141678624535 | +0.000004303461 |

### Actual mode differences

| Pair | Different comparison records | Different predicted IDs | Different target probabilities |
|---|---:|---:|---:|
| FF_vs_QF | 233472 | 38583 | 233472 |
| FF_vs_FQ | 233472 | 452 | 233433 |
| FF_vs_QQ | 233472 | 38592 | 233472 |
| QF_vs_FQ | 233472 | 38590 | 233472 |
| QF_vs_QQ | 233472 | 437 | 233430 |
| FQ_vs_QQ | 233472 | 38603 | 233471 |

### Complete free generations

The text below is the complete recorded raw decoded continuation. Stop reasons and token IDs are retained; no output repair or truncation is applied.

#### Prompt 1

```text
Once upon a time, there was a mommy and daddy playing with their 3-year-old son. Mommy
```

**FF**, seed2014, stop={"reason": "maximum_new_tokens"}, UTF8=True

```text
 had a very special room with lots of paper. She loved lots of things she loved. She put on her clothes and went outside and walked in the kitchen.
At the room, Mommy saw Anna and Lily, who were fighting. Mommy was upset, and her daddy asked her what Anna was doing.
"What is that, Anna?" daddy asked.
 the paper shook her head. "It's very special to Anna and no! I need to wash it up!". Anna was very happy to be in the room.
The paper was very hot and warm. Anna was so happy. She held the paper with her daddy
```

Generated token IDs: `[366, 261, 381, 854, 710, 331, 852, 372, 1323, 16, 341, 504, 852, 372, 626, 394, 504, 16, 341, 572, 356, 338, 1571, 269, 430, 699, 269, 784, 315, 265, 1170, 16, 201, 1229, 265, 710, 14, 1522, 384, 894, 269, 407, 14, 660, 406, 2213, 16, 1522, 285, 2077, 14, 269, 338, 2218, 548, 338, 579, 894, 285, 1452, 16, 201, 4, 1085, 435, 383, 14, 894, 478, 2218, 548, 16, 201, 265, 1323, 1994, 338, 1153, 16, 319, 1020, 386, 381, 854, 268, 894, 269, 542, 3, 340, 767, 268, 2341, 311, 368, 415, 16, 894, 285, 381, 378, 268, 324, 315, 265, 710, 16, 201, 413, 1323, 285, 381, 1254, 269, 1057, 16, 894, 285, 391, 378, 16, 341, 1444, 265, 1323, 331, 338, 2218]`

Recorded response_text (as separately stored by the evaluator):

```text
had a very special room with lots of paper. She loved lots of things she loved. She put on her clothes and went outside and walked in the kitchen.
At the room, Mommy saw Anna and Lily, who were fighting. Mommy was upset, and her daddy asked her what Anna was doing.
"What is that, Anna?" daddy asked.
 the paper shook her head. "It's very special to Anna and no! I need to wash it up!". Anna was very happy to be in the room.
The paper was very hot and warm. Anna was so happy. She held the paper with her daddy
```

**QF**, seed2014, stop={"reason": "eos"}, UTF8=True

```text
 and daddy had a very special day.<|eos|>
```

Generated token IDs: `[269, 2218, 366, 261, 381, 854, 329, 16, 1]`

Recorded response_text (as separately stored by the evaluator):

```text
and daddy had a very special day.
```

**FQ**, seed2014, stop={"reason": "maximum_new_tokens"}, UTF8=True

```text
 had a very special room with lots of paper. She loved lots of things she loved. She put on her clothes and went outside and closed the door.
One day, the mommy went to daddy and dad. She had a big red box and many books inside. She was very curious and wanted to see what was inside. So, she stepped closer and saw something beautiful. It was a big, round red ball.
The next day, she went outside to play. He heard a voice say: "Hey, I am scared of a red ball. Can we play with it?" The mommy smiled and said, "Of course
```

Generated token IDs: `[366, 261, 381, 854, 710, 331, 852, 372, 1323, 16, 341, 504, 852, 372, 626, 394, 504, 16, 341, 572, 356, 338, 1571, 269, 430, 699, 269, 1772, 265, 888, 16, 201, 385, 329, 14, 265, 1152, 430, 268, 2218, 269, 728, 16, 341, 366, 261, 348, 677, 616, 269, 664, 1455, 747, 16, 341, 285, 381, 1208, 269, 409, 268, 538, 579, 285, 747, 16, 712, 14, 394, 2254, 1302, 269, 384, 577, 997, 16, 427, 285, 261, 348, 14, 1623, 677, 481, 16, 201, 413, 984, 329, 14, 394, 430, 699, 268, 327, 16, 318, 827, 261, 1276, 562, 28, 319, 2172, 14, 340, 745, 611, 372, 261, 677, 481, 16, 1143, 365, 327, 331, 311, 478, 287, 1152, 565, 269, 326, 14, 319, 2181, 1984]`

Recorded response_text (as separately stored by the evaluator):

```text
had a very special room with lots of paper. She loved lots of things she loved. She put on her clothes and went outside and closed the door.
One day, the mommy went to daddy and dad. She had a big red box and many books inside. She was very curious and wanted to see what was inside. So, she stepped closer and saw something beautiful. It was a big, round red ball.
The next day, she went outside to play. He heard a voice say: "Hey, I am scared of a red ball. Can we play with it?" The mommy smiled and said, "Of course
```

**QQ**, seed2014, stop={"reason": "eos"}, UTF8=True

```text
 and daddy had a very special day.<|eos|>
```

Generated token IDs: `[269, 2218, 366, 261, 381, 854, 329, 16, 1]`

Recorded response_text (as separately stored by the evaluator):

```text
and daddy had a very special day.
```

#### Prompt 2

```text
Once upon a time, there was a mighty lion walking through the jungle. He was looking for his friends, and he
```

**FF**, seed2015, stop={"reason": "maximum_new_tokens"}, UTF8=True

```text
 felt a bit anxious. He looked out the window and saw a big tree.
"Maybe if someone touched it," he said.
The mighty lion was scared. He wanted to stay away from it. He had his friends. He thought the jungle was very nice and exciting.
He saw a big truck coming and said, "Let's go to the truck and get it!"
The mighty lion and his friends had to keep the truck as a little boy who was sitting on. He was so scared. He ran around the jungle and could not get it anywhere.
But he was too mighty to go inside. He
```

Generated token IDs: `[535, 261, 1435, 3161, 16, 318, 506, 486, 265, 1413, 269, 384, 261, 348, 497, 16, 201, 4, 1718, 837, 1696, 2013, 311, 638, 288, 326, 16, 201, 413, 2679, 1296, 285, 611, 16, 318, 409, 268, 963, 575, 583, 311, 16, 318, 366, 343, 411, 16, 318, 585, 265, 3191, 285, 381, 732, 269, 2935, 16, 201, 1035, 384, 261, 348, 1043, 1433, 269, 326, 14, 319, 897, 386, 480, 268, 265, 1043, 269, 646, 311, 415, 201, 413, 2679, 1296, 269, 343, 411, 366, 268, 1040, 265, 1043, 450, 261, 393, 501, 660, 285, 1807, 356, 16, 318, 285, 391, 611, 16, 318, 594, 622, 265, 3191, 269, 468, 367, 646, 311, 672, 1358, 16, 201, 769, 288, 285, 425, 2679, 268, 480, 747, 16, 318]`

Recorded response_text (as separately stored by the evaluator):

```text
felt a bit anxious. He looked out the window and saw a big tree.
"Maybe if someone touched it," he said.
The mighty lion was scared. He wanted to stay away from it. He had his friends. He thought the jungle was very nice and exciting.
He saw a big truck coming and said, "Let's go to the truck and get it!"
The mighty lion and his friends had to keep the truck as a little boy who was sitting on. He was so scared. He ran around the jungle and could not get it anywhere.
But he was too mighty to go inside. He
```

**QF**, seed2015, stop={"reason": "eos"}, UTF8=True

```text
 needed to take off the jungle. The lion was very sad and he did not like that his friends, so he saw his friends and they were never able to play again soon.
The lion was his friends and played every day. He had his best friends and played together every day. He was very happy to have a new friends to play and have fun.<|eos|>
```

Generated token IDs: `[1291, 268, 754, 938, 265, 3191, 16, 287, 1296, 285, 381, 496, 269, 288, 454, 367, 517, 383, 343, 411, 14, 391, 288, 384, 343, 411, 269, 364, 406, 856, 1964, 268, 327, 602, 1084, 16, 201, 413, 1296, 285, 343, 411, 269, 476, 589, 329, 16, 318, 366, 343, 688, 411, 269, 476, 458, 589, 329, 16, 318, 285, 381, 378, 268, 500, 261, 545, 411, 268, 327, 269, 500, 446, 16, 1]`

Recorded response_text (as separately stored by the evaluator):

```text
needed to take off the jungle. The lion was very sad and he did not like that his friends, so he saw his friends and they were never able to play again soon.
The lion was his friends and played every day. He had his best friends and played together every day. He was very happy to have a new friends to play and have fun.
```

**FQ**, seed2015, stop={"reason": "maximum_new_tokens"}, UTF8=True

```text
 felt a bit anxious. He looked out the window and saw a big tree.
"Maybe if someone touched it," he said.
The mighty lion was scared. He wanted to stay away from it. He had his friends. He thought the jungle was very nice and exciting.
He saw a big truck coming and said, "Let's go to the truck and get it!"
The mighty lion and his friends had to keep the truck as a little boy who was sitting on. He was so scared. He ran around the jungle and hid behind a big tree. Then, the lion got up and said, "I am
```

Generated token IDs: `[535, 261, 1435, 3161, 16, 318, 506, 486, 265, 1413, 269, 384, 261, 348, 497, 16, 201, 4, 1718, 837, 1696, 2013, 311, 638, 288, 326, 16, 201, 413, 2679, 1296, 285, 611, 16, 318, 409, 268, 963, 575, 583, 311, 16, 318, 366, 343, 411, 16, 318, 585, 265, 3191, 285, 381, 732, 269, 2935, 16, 201, 1035, 384, 261, 348, 1043, 1433, 269, 326, 14, 319, 897, 386, 480, 268, 265, 1043, 269, 646, 311, 415, 201, 413, 2679, 1296, 269, 343, 411, 366, 268, 1040, 265, 1043, 450, 261, 393, 501, 660, 285, 1807, 356, 16, 318, 285, 391, 611, 16, 318, 594, 622, 265, 3191, 269, 1218, 1268, 261, 348, 497, 16, 872, 14, 265, 1296, 659, 368, 269, 326, 14, 319, 43, 745]`

Recorded response_text (as separately stored by the evaluator):

```text
felt a bit anxious. He looked out the window and saw a big tree.
"Maybe if someone touched it," he said.
The mighty lion was scared. He wanted to stay away from it. He had his friends. He thought the jungle was very nice and exciting.
He saw a big truck coming and said, "Let's go to the truck and get it!"
The mighty lion and his friends had to keep the truck as a little boy who was sitting on. He was so scared. He ran around the jungle and hid behind a big tree. Then, the lion got up and said, "I am
```

**QQ**, seed2015, stop={"reason": "eos"}, UTF8=True

```text
 needed to take off the jungle. The lion was very sad and he did not like that his friends, so he saw his friends and they were never able to play again soon.
The lion was his friends and were always happy to help. The jungle was his friends and everyone was very happy. The lion was not sad anymore, and he lived happily ever after.<|eos|>
```

Generated token IDs: `[1291, 268, 754, 938, 265, 3191, 16, 287, 1296, 285, 381, 496, 269, 288, 454, 367, 517, 383, 343, 411, 14, 391, 288, 384, 343, 411, 269, 364, 406, 856, 1964, 268, 327, 602, 1084, 16, 201, 413, 1296, 285, 343, 411, 269, 406, 701, 378, 268, 436, 16, 287, 3191, 285, 343, 411, 269, 1014, 285, 381, 378, 16, 287, 1296, 285, 367, 496, 858, 14, 269, 288, 630, 987, 929, 902, 16, 1]`

Recorded response_text (as separately stored by the evaluator):

```text
needed to take off the jungle. The lion was very sad and he did not like that his friends, so he saw his friends and they were never able to play again soon.
The lion was his friends and were always happy to help. The jungle was his friends and everyone was very happy. The lion was not sad anymore, and he lived happily ever after.
```

#### Prompt 3

```text
Once there was a pink goose. She was very pretty. The goose liked to eat grass, corn and bugs. One
```

**FF**, seed2016, stop={"reason": "eos"}, UTF8=True

```text
 day, it went into the woods. It was very special. The goose liked to jump around and play in the woods.
The goose was feeling guilty. The goose thought to say, "You can't be at the woods, but be careful not to step too high for you to play in."
The goose felt happy that he won the woods. The goose was happy to have to be careful with the goose. And they all lived happily ever after.<|eos|>
```

Generated token IDs: `[329, 14, 311, 430, 719, 265, 1507, 16, 427, 285, 381, 854, 16, 287, 2922, 531, 268, 721, 622, 269, 327, 315, 265, 1507, 16, 201, 413, 2922, 285, 1314, 3181, 16, 287, 2922, 585, 268, 562, 14, 319, 536, 469, 488, 324, 452, 265, 1507, 14, 412, 324, 846, 367, 268, 2286, 425, 901, 388, 351, 268, 327, 315, 399, 201, 413, 2922, 535, 378, 383, 288, 1611, 265, 1507, 16, 287, 2922, 285, 378, 268, 500, 268, 324, 846, 331, 265, 2922, 16, 715, 364, 434, 630, 987, 929, 902, 16, 1]`

Recorded response_text (as separately stored by the evaluator):

```text
day, it went into the woods. It was very special. The goose liked to jump around and play in the woods.
The goose was feeling guilty. The goose thought to say, "You can't be at the woods, but be careful not to step too high for you to play in."
The goose felt happy that he won the woods. The goose was happy to have to be careful with the goose. And they all lived happily ever after.
```

**QF**, seed2016, stop={"reason": "maximum_new_tokens"}, UTF8=True

```text
 day, it was hot and cold. The goose wanted to go outside and play with their goose.
While the goose went outside, the goose noticed something strange. It was not real! There were many bugs in a big tree. The goose was surprised.
The goose decided to eat the bugs and play with their goose. They had an idea. The goose said, "I will hide for you, goose!" The goose was happy and safe.
The goose was so happy that the bugs could play with the bugs in the tree. The goose learned that it is the best thing to need to eat for your goose. And that
```

Generated token IDs: `[329, 14, 311, 285, 1254, 269, 1214, 16, 287, 2922, 409, 268, 480, 699, 269, 327, 331, 470, 2922, 16, 201, 1766, 265, 2922, 430, 699, 14, 265, 2922, 1553, 577, 1834, 16, 427, 285, 367, 1192, 3, 1526, 406, 664, 2030, 315, 261, 348, 497, 16, 287, 2922, 285, 900, 16, 201, 413, 2922, 709, 268, 711, 265, 2030, 269, 327, 331, 470, 2922, 16, 314, 366, 423, 822, 16, 287, 2922, 326, 14, 319, 43, 619, 1438, 388, 351, 14, 2922, 415, 287, 2922, 285, 378, 269, 899, 16, 201, 413, 2922, 285, 391, 378, 383, 265, 2030, 468, 327, 331, 265, 2030, 315, 265, 497, 16, 287, 2922, 694, 383, 311, 435, 265, 688, 1244, 268, 767, 268, 711, 388, 631, 2922, 16, 715, 383]`

Recorded response_text (as separately stored by the evaluator):

```text
day, it was hot and cold. The goose wanted to go outside and play with their goose.
While the goose went outside, the goose noticed something strange. It was not real! There were many bugs in a big tree. The goose was surprised.
The goose decided to eat the bugs and play with their goose. They had an idea. The goose said, "I will hide for you, goose!" The goose was happy and safe.
The goose was so happy that the bugs could play with the bugs in the tree. The goose learned that it is the best thing to need to eat for your goose. And that
```

**FQ**, seed2016, stop={"reason": "eos"}, UTF8=True

```text
 day, it went into the woods. It was very special. The goose liked to jump around and play in the woods.
The goose was feeling guilty. The goose thought to say, "You can't be at the woods, but be careful not to step too high for you to play in."
The goose felt happy that he won the woods. The goose was happy to have his friend.<|eos|>
```

Generated token IDs: `[329, 14, 311, 430, 719, 265, 1507, 16, 427, 285, 381, 854, 16, 287, 2922, 531, 268, 721, 622, 269, 327, 315, 265, 1507, 16, 201, 413, 2922, 285, 1314, 3181, 16, 287, 2922, 585, 268, 562, 14, 319, 536, 469, 488, 324, 452, 265, 1507, 14, 412, 324, 846, 367, 268, 2286, 425, 901, 388, 351, 268, 327, 315, 399, 201, 413, 2922, 535, 378, 383, 288, 1611, 265, 1507, 16, 287, 2922, 285, 378, 268, 500, 343, 360, 16, 1]`

Recorded response_text (as separately stored by the evaluator):

```text
day, it went into the woods. It was very special. The goose liked to jump around and play in the woods.
The goose was feeling guilty. The goose thought to say, "You can't be at the woods, but be careful not to step too high for you to play in."
The goose felt happy that he won the woods. The goose was happy to have his friend.
```

**QQ**, seed2016, stop={"reason": "maximum_new_tokens"}, UTF8=True

```text
 day, it was hot and cold. The goose wanted to go outside and play with their goose.
While the goose went outside, the goose noticed something strange. It was not real! There were many trees, so many birds, corners, and even big. They tried to get the birds, but the goose was too big. So, the goose decided to eat the birds. The goose was very happy and could not wait to play with them all.
The next day, the goose went outside. It was too big for the goose to eat. The goose did not like the dog. The goose was sad and hungry.
```

Generated token IDs: `[329, 14, 311, 285, 1254, 269, 1214, 16, 287, 2922, 409, 268, 480, 699, 269, 327, 331, 470, 2922, 16, 201, 1766, 265, 2922, 430, 699, 14, 265, 2922, 1553, 577, 1834, 16, 427, 285, 367, 1192, 3, 1526, 406, 664, 1253, 14, 391, 664, 1118, 14, 1968, 720, 14, 269, 868, 348, 16, 314, 658, 268, 646, 265, 1118, 14, 412, 265, 2922, 285, 425, 348, 16, 712, 14, 265, 2922, 709, 268, 711, 265, 1118, 16, 287, 2922, 285, 381, 378, 269, 468, 367, 1168, 268, 327, 331, 495, 434, 16, 201, 413, 984, 329, 14, 265, 2922, 430, 699, 16, 427, 285, 425, 348, 388, 265, 2922, 268, 711, 16, 287, 2922, 454, 367, 517, 265, 466, 16, 287, 2922, 285, 496, 269, 1343, 16]`

Recorded response_text (as separately stored by the evaluator):

```text
day, it was hot and cold. The goose wanted to go outside and play with their goose.
While the goose went outside, the goose noticed something strange. It was not real! There were many trees, so many birds, corners, and even big. They tried to get the birds, but the goose was too big. So, the goose decided to eat the birds. The goose was very happy and could not wait to play with them all.
The next day, the goose went outside. It was too big for the goose to eat. The goose did not like the dog. The goose was sad and hungry.
```

#### Prompt 4

```text
One day, a boy named Tim wanted to ride his bike. He loved to ride up the big hill near his house
```

**FF**, seed2017, stop={"reason": "maximum_new_tokens"}, UTF8=True

```text
. The bike was not boring anymore, but Tim liked to play.
Tim went to the park with his mom. He saw a big tree and wanted to climb it. But Tim was scared, but he stayed in the tree. The bike went to the tree and started to climb the tree. As he climbed higher, the bike fell loo. Tim was scared.
They rolled up the tree and saw the big hole in the tree.
Suddenly, a big wind came and blew the tree far away. Tim and his mom were sad. They did not know what to do. The wind blew harder to get out. But the
```

Generated token IDs: `[16, 287, 1584, 285, 367, 2465, 858, 14, 412, 344, 531, 268, 327, 16, 201, 649, 430, 268, 265, 524, 331, 343, 390, 16, 318, 384, 261, 348, 497, 269, 409, 268, 884, 311, 16, 543, 344, 285, 611, 14, 412, 288, 1529, 315, 265, 497, 16, 287, 1584, 430, 268, 265, 497, 269, 549, 268, 884, 265, 497, 16, 1336, 288, 1331, 1857, 14, 265, 1584, 1064, 437, 16, 344, 285, 611, 16, 201, 725, 1759, 368, 265, 497, 269, 384, 265, 348, 1089, 315, 265, 497, 16, 201, 1380, 14, 261, 348, 864, 553, 269, 1540, 265, 497, 932, 575, 16, 344, 269, 343, 390, 406, 496, 16, 314, 454, 367, 657, 579, 268, 377, 16, 287, 864, 1540, 3192, 268, 646, 486, 16, 543, 265]`

**QF**, seed2017, stop={"reason": "maximum_new_tokens"}, UTF8=True

```text
.
As Tim rode, the hill was dark. Tim saw his friend, Sue, came over to play. Tim said, "Wow, I like your bike!" Sue said, "Wow! I like a bike too!" They played together, and Tim was very happy, and they played together with their bike.
But then, we like to go back to the beach and have fun!" Tim said. They ran and jumped in the sun, but then they heard a voice. "Quick, let's go!" said Tim. Sue agreed and they started to play. They ran and ran and had lots of fun.
```

Generated token IDs: `[16, 201, 921, 344, 3272, 14, 265, 1202, 285, 1269, 16, 344, 384, 343, 360, 14, 510, 14, 553, 883, 268, 327, 16, 344, 326, 14, 319, 1256, 14, 340, 517, 631, 1584, 415, 510, 326, 14, 319, 1256, 3, 340, 517, 261, 1584, 425, 415, 314, 476, 458, 14, 269, 344, 285, 381, 378, 14, 269, 364, 476, 458, 331, 470, 1584, 16, 201, 769, 634, 14, 365, 517, 268, 480, 515, 268, 265, 1699, 269, 500, 446, 415, 344, 326, 16, 314, 594, 269, 945, 315, 265, 629, 14, 412, 634, 364, 827, 261, 1276, 16, 319, 51, 87, 618, 14, 832, 386, 480, 415, 326, 344, 16, 510, 1251, 269, 364, 549, 268, 327, 16, 314, 594, 269, 594, 269, 366, 852, 372, 446, 16]`

**FQ**, seed2017, stop={"reason": "maximum_new_tokens"}, UTF8=True

```text
. The bike was not boring anymore, but Tim liked to play.
Tim went to the park with his mom. He saw a big tree and wanted to climb it. But Tim was scared, but he stayed in the tree. The bike went to the tree and started to climb the tree. As he climbed higher, the bike fell loo. Tim was scared.
They rolled up the tree and saw the big hole in the tree.
Suddenly, a big wind came and blew the tree far away. Tim and his mom were sad. They did not know what to do. The wind blew away a hard way. Tim and
```

Generated token IDs: `[16, 287, 1584, 285, 367, 2465, 858, 14, 412, 344, 531, 268, 327, 16, 201, 649, 430, 268, 265, 524, 331, 343, 390, 16, 318, 384, 261, 348, 497, 269, 409, 268, 884, 311, 16, 543, 344, 285, 611, 14, 412, 288, 1529, 315, 265, 497, 16, 287, 1584, 430, 268, 265, 497, 269, 549, 268, 884, 265, 497, 16, 1336, 288, 1331, 1857, 14, 265, 1584, 1064, 437, 16, 344, 285, 611, 16, 201, 725, 1759, 368, 265, 497, 269, 384, 265, 348, 1089, 315, 265, 497, 16, 201, 1380, 14, 261, 348, 864, 553, 269, 1540, 265, 497, 932, 575, 16, 344, 269, 343, 390, 406, 496, 16, 314, 454, 367, 657, 579, 268, 377, 16, 287, 864, 1540, 575, 261, 881, 916, 16, 344, 269]`

**QQ**, seed2017, stop={"reason": "maximum_new_tokens"}, UTF8=True

```text
.
As Tim rode, the hill was dark. Tim saw his friend, Sue, came over to play. Tim said, "Wow, I like your bike!" Sue said, "Wow! I like a bike too!" They played together, and Tim was very happy, and they played together with their bike.
But then, we like to go back to the beach and have fun!" Tim said. They ran and jumped in the sun, but then they heard a voice. "Quick, let's go!" said Tim. Sue agreed and they started to play. They ran and ran and had lots of fun.
```

Generated token IDs: `[16, 201, 921, 344, 3272, 14, 265, 1202, 285, 1269, 16, 344, 384, 343, 360, 14, 510, 14, 553, 883, 268, 327, 16, 344, 326, 14, 319, 1256, 14, 340, 517, 631, 1584, 415, 510, 326, 14, 319, 1256, 3, 340, 517, 261, 1584, 425, 415, 314, 476, 458, 14, 269, 344, 285, 381, 378, 14, 269, 364, 476, 458, 331, 470, 1584, 16, 201, 769, 634, 14, 365, 517, 268, 480, 515, 268, 265, 1699, 269, 500, 446, 415, 344, 326, 16, 314, 594, 269, 945, 315, 265, 629, 14, 412, 634, 364, 827, 261, 1276, 16, 319, 51, 87, 618, 14, 832, 386, 480, 415, 326, 344, 16, 510, 1251, 269, 364, 549, 268, 327, 16, 314, 594, 269, 594, 269, 366, 852, 372, 446, 16]`

#### Prompt 5

```text
One day, there was a small and modest train. Inside the train were two friends - Lily and Dave. They
```

**FF**, seed2018, stop={"reason": "maximum_new_tokens"}, UTF8=True

```text
 would take to the park and play.
One day, Lily and Dave went to the park. He saw a big tree with many branches. The tree was very small and had a hole in it. The train was sad and wanted to play with it too. So, they rolled it down and down, and the train was too modest.
As they played, the train got very dark. Dave's eyes got hurt, so he cried.
The train was too strong and it fell off, and the train fell down. Dave and Dave were sad but they all played together. They were very happy and played
```

Generated token IDs: `[527, 754, 268, 265, 524, 269, 327, 16, 201, 385, 329, 14, 407, 269, 671, 605, 430, 268, 265, 524, 16, 318, 384, 261, 348, 497, 331, 664, 2710, 16, 287, 497, 285, 381, 563, 269, 366, 261, 1089, 315, 311, 16, 287, 1504, 285, 496, 269, 409, 268, 327, 331, 311, 425, 16, 712, 14, 364, 1759, 311, 601, 269, 601, 14, 269, 265, 1504, 285, 425, 3713, 16, 201, 921, 364, 476, 14, 265, 1504, 659, 381, 1269, 16, 671, 605, 386, 1162, 659, 896, 14, 391, 288, 1322, 16, 201, 413, 1504, 285, 425, 976, 269, 311, 1064, 938, 14, 269, 265, 1504, 1064, 601, 16, 671, 605, 269, 671, 605, 406, 496, 412, 364, 434, 476, 458, 16, 314, 406, 381, 378, 269, 476]`

Recorded response_text (as separately stored by the evaluator):

```text
would take to the park and play.
One day, Lily and Dave went to the park. He saw a big tree with many branches. The tree was very small and had a hole in it. The train was sad and wanted to play with it too. So, they rolled it down and down, and the train was too modest.
As they played, the train got very dark. Dave's eyes got hurt, so he cried.
The train was too strong and it fell off, and the train fell down. Dave and Dave were sad but they all played together. They were very happy and played
```

**QF**, seed2018, stop={"reason": "maximum_new_tokens"}, UTF8=True

```text
 played in a big house. They wanted to see their friends and family. Lily and Dave were happy in their house.
In the end, Lily and Dave went to the park. They saw a big hill. The train was very tall and could lift. They went on the swings and the train. The train was very strong.
One day, Dave found a new friend, a bird named Tim. Tim was happy. He wanted to lift the train too. So, Dave asked him, "Can I lift the train, please?"
The train driver said, "No, I want to lift the train
```

Generated token IDs: `[476, 315, 261, 348, 603, 16, 314, 409, 268, 538, 470, 411, 269, 951, 16, 407, 269, 671, 605, 406, 378, 315, 470, 603, 16, 201, 1469, 265, 866, 14, 407, 269, 671, 605, 430, 268, 265, 524, 16, 314, 384, 261, 348, 1202, 16, 287, 1504, 285, 381, 1243, 269, 468, 2342, 16, 314, 430, 356, 265, 1889, 269, 265, 1504, 16, 287, 1504, 285, 381, 976, 16, 201, 385, 329, 14, 671, 605, 505, 261, 545, 360, 14, 261, 422, 404, 344, 16, 344, 285, 378, 16, 318, 409, 268, 2342, 265, 1504, 425, 16, 712, 14, 671, 605, 548, 474, 14, 319, 1038, 340, 2342, 265, 1504, 14, 1293, 478, 201, 413, 1504, 2141, 326, 14, 319, 937, 14, 340, 371, 268, 2342, 265, 1504]`

Recorded response_text (as separately stored by the evaluator):

```text
played in a big house. They wanted to see their friends and family. Lily and Dave were happy in their house.
In the end, Lily and Dave went to the park. They saw a big hill. The train was very tall and could lift. They went on the swings and the train. The train was very strong.
One day, Dave found a new friend, a bird named Tim. Tim was happy. He wanted to lift the train too. So, Dave asked him, "Can I lift the train, please?"
The train driver said, "No, I want to lift the train
```

**FQ**, seed2018, stop={"reason": "maximum_new_tokens"}, UTF8=True

```text
 would take to the park and play.
One day, Lily and Dave went to the park. At the park, Lily and Dave went outside. They saw a big tree and wanted to climb it. Dave said, "Let's go!" The train looked at the price and they both laughed.
They all took turns to climb the tree. Lily was so happy and excited. She ran around the train and pushed them up. They played and laughed. But then, something unexpected happened! The train started to shake. The train said, "I am making a loud noise!" Lily and Dave were surprised.
```

Generated token IDs: `[527, 754, 268, 265, 524, 269, 327, 16, 201, 385, 329, 14, 407, 269, 671, 605, 430, 268, 265, 524, 16, 1723, 265, 524, 14, 407, 269, 671, 605, 430, 699, 16, 314, 384, 261, 348, 497, 269, 409, 268, 884, 311, 16, 671, 605, 326, 14, 319, 897, 386, 480, 415, 287, 1504, 506, 452, 265, 2550, 339, 269, 364, 717, 705, 16, 201, 725, 434, 614, 1579, 268, 884, 265, 497, 16, 407, 285, 391, 378, 269, 726, 16, 341, 594, 622, 265, 1504, 269, 1443, 495, 368, 16, 314, 476, 269, 705, 16, 543, 634, 14, 577, 913, 807, 3, 287, 1504, 549, 268, 2024, 16, 287, 1504, 326, 14, 319, 43, 745, 1215, 261, 911, 1066, 415, 407, 269, 671, 605, 406, 900, 16]`

Recorded response_text (as separately stored by the evaluator):

```text
would take to the park and play.
One day, Lily and Dave went to the park. At the park, Lily and Dave went outside. They saw a big tree and wanted to climb it. Dave said, "Let's go!" The train looked at the price and they both laughed.
They all took turns to climb the tree. Lily was so happy and excited. She ran around the train and pushed them up. They played and laughed. But then, something unexpected happened! The train started to shake. The train said, "I am making a loud noise!" Lily and Dave were surprised.
```

**QQ**, seed2018, stop={"reason": "maximum_new_tokens"}, UTF8=True

```text
 played in a big house. They wanted to see their friends and family. Lily and Dave were happy in their house.
In the end, Lily and Dave went to the park. They saw a big hill. The train was very tall and could lift. They went on the swings and the train. The train was very strong.
One day, Dave found a new friend, a bird named Tim. Tim was too excited to help the bird. He asked the bird, "Can I do you want to play with you?" The bird said, "Yes, you can play with me if you help me lift the
```

Generated token IDs: `[476, 315, 261, 348, 603, 16, 314, 409, 268, 538, 470, 411, 269, 951, 16, 407, 269, 671, 605, 406, 378, 315, 470, 603, 16, 201, 1469, 265, 866, 14, 407, 269, 671, 605, 430, 268, 265, 524, 16, 314, 384, 261, 348, 1202, 16, 287, 1504, 285, 381, 1243, 269, 468, 2342, 16, 314, 430, 356, 265, 1889, 269, 265, 1504, 16, 287, 1504, 285, 381, 976, 16, 201, 385, 329, 14, 671, 605, 505, 261, 545, 360, 14, 261, 422, 404, 344, 16, 344, 285, 425, 726, 268, 436, 265, 422, 16, 318, 548, 265, 422, 14, 319, 1038, 340, 377, 351, 371, 268, 327, 331, 351, 478, 287, 422, 326, 14, 319, 735, 14, 351, 469, 327, 331, 519, 837, 351, 436, 519, 2342, 265]`

Recorded response_text (as separately stored by the evaluator):

```text
played in a big house. They wanted to see their friends and family. Lily and Dave were happy in their house.
In the end, Lily and Dave went to the park. They saw a big hill. The train was very tall and could lift. They went on the swings and the train. The train was very strong.
One day, Dave found a new friend, a bird named Tim. Tim was too excited to help the bird. He asked the bird, "Can I do you want to play with you?" The bird said, "Yes, you can play with me if you help me lift the
```

### Source-probe rows

| Mode | First noun correct /32 | Complete correct /32 |
|---|---:|---:|
| FF | 31 | 28 |
| QF | 31 | 22 |
| FQ | 31 | 28 |
| QQ | 31 | 22 |

#### story-source-edit-00 / original

Accepted continuations: `["apple."]`

```text
One day, Lily carried her apple into the garden. She put it on the little table beside the door. Then she ran to play with her friend. They laughed and watched the clouds move across the sky. When it was time to go home, Lily went back to the table and picked up the
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 apple.
```

Recorded response_text:

```text
apple.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 apple.
```

Recorded response_text:

```text
apple.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 apple.
```

Recorded response_text:

```text
apple.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 apple.
```

Recorded response_text:

```text
apple.
```

#### story-source-edit-00 / edited

Accepted continuations: `["pear."]`

```text
One day, Lily carried her pear into the garden. She put it on the little table beside the door. Then she ran to play with her friend. They laughed and watched the clouds move across the sky. When it was time to go home, Lily went back to the table and picked up the
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 pear.
```

Recorded response_text:

```text
pear.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 pear.
```

Recorded response_text:

```text
pear.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 pear.
```

Recorded response_text:

```text
pear.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 pear.
```

Recorded response_text:

```text
pear.
```

#### story-source-edit-01 / original

Accepted continuations: `["ball."]`

```text
Tim wanted to keep his ball safe. He put it under the wooden bench before he went outside. He walked around the pond and waved to the ducks. Soon his mother called him home for lunch. Tim remembered what he had left behind. He looked under the bench and found his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 ball.
```

Recorded response_text:

```text
ball.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 ball.
```

Recorded response_text:

```text
ball.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 ball.
```

Recorded response_text:

```text
ball.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 ball.
```

Recorded response_text:

```text
ball.
```

#### story-source-edit-01 / edited

Accepted continuations: `["kite."]`

```text
Tim wanted to keep his kite safe. He put it under the wooden bench before he went outside. He walked around the pond and waved to the ducks. Soon his mother called him home for lunch. Tim remembered what he had left behind. He looked under the bench and found his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 kite.
```

Recorded response_text:

```text
kite.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 kite.
```

Recorded response_text:

```text
kite.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 kite.
```

Recorded response_text:

```text
kite.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 kite.
```

Recorded response_text:

```text
kite.
```

#### story-source-edit-02 / original

Accepted continuations: `["book."]`

```text
Anna brought her book to the park. She left it beside a tall tree while she played with her brother. They ran along the path and listened to the birds. The sun began to set, and they were ready to leave. Anna returned to the tree to collect her
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 book.
```

Recorded response_text:

```text
book.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 book.
```

Recorded response_text:

```text
book.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 book.
```

Recorded response_text:

```text
book.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 book.
```

Recorded response_text:

```text
book.
```

#### story-source-edit-02 / edited

Accepted continuations: `["toy."]`

```text
Anna brought her toy to the park. She left it beside a tall tree while she played with her brother. They ran along the path and listened to the birds. The sun began to set, and they were ready to leave. Anna returned to the tree to collect her
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 toy.
```

Recorded response_text:

```text
toy.
```

**QF**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 toy to play with her toy.
```

Recorded response_text:

```text
toy to play with her toy.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 toy.
```

Recorded response_text:

```text
toy.
```

**QQ**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 toy to play with her toy.
```

Recorded response_text:

```text
toy to play with her toy.
```

#### story-source-edit-03 / original

Accepted continuations: `["bell."]`

```text
Ben was very happy with his bell. Before dinner, he placed it on the kitchen chair. He washed his hands and helped his father with the plates. After they had finished eating, Ben remembered his special thing. He went to the chair and took his
```

**FF**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 bell to the kitchen.
```

Recorded response_text:

```text
bell to the kitchen.
```

**QF**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 bell to the kitchen.
```

Recorded response_text:

```text
bell to the kitchen.
```

**FQ**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 bell to the kitchen.
```

Recorded response_text:

```text
bell to the kitchen.
```

**QQ**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 bell to the kitchen.
```

Recorded response_text:

```text
bell to the kitchen.
```

#### story-source-edit-03 / edited

Accepted continuations: `["drum."]`

```text
Ben was very happy with his drum. Before dinner, he placed it on the kitchen chair. He washed his hands and helped his father with the plates. After they had finished eating, Ben remembered his special thing. He went to the chair and took his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 drum.
```

Recorded response_text:

```text
drum.
```

**QF**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 drum to the kitchen.
```

Recorded response_text:

```text
drum to the kitchen.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 drum.
```

Recorded response_text:

```text
drum.
```

**QQ**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 drum to the kitchen.
```

Recorded response_text:

```text
drum to the kitchen.
```

#### story-source-edit-04 / original

Accepted continuations: `["doll."]`

```text
One day, Lily carried her doll into the garden. She put it on the little table beside the door. Then she ran to play with her friend. They laughed and watched the clouds move across the sky. When it was time to go home, Lily went back to the table and picked up the
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 doll.
```

Recorded response_text:

```text
doll.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 doll.
```

Recorded response_text:

```text
doll.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 doll.
```

Recorded response_text:

```text
doll.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 doll.
```

Recorded response_text:

```text
doll.
```

#### story-source-edit-04 / edited

Accepted continuations: `["bear."]`

```text
One day, Lily carried her bear into the garden. She put it on the little table beside the door. Then she ran to play with her friend. They laughed and watched the clouds move across the sky. When it was time to go home, Lily went back to the table and picked up the
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 bear.
```

Recorded response_text:

```text
bear.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 bear.
```

Recorded response_text:

```text
bear.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 bear.
```

Recorded response_text:

```text
bear.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 bear.
```

Recorded response_text:

```text
bear.
```

#### story-source-edit-05 / original

Accepted continuations: `["cup."]`

```text
Tim wanted to keep his cup safe. He put it under the wooden bench before he went outside. He walked around the pond and waved to the ducks. Soon his mother called him home for lunch. Tim remembered what he had left behind. He looked under the bench and found his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 cup.
```

Recorded response_text:

```text
cup.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 cup.
```

Recorded response_text:

```text
cup.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 cup.
```

Recorded response_text:

```text
cup.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 cup.
```

Recorded response_text:

```text
cup.
```

#### story-source-edit-05 / edited

Accepted continuations: `["bowl."]`

```text
Tim wanted to keep his bowl safe. He put it under the wooden bench before he went outside. He walked around the pond and waved to the ducks. Soon his mother called him home for lunch. Tim remembered what he had left behind. He looked under the bench and found his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 bowl.
```

Recorded response_text:

```text
bowl.
```

**QF**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 favorite toy.
```

Recorded response_text:

```text
favorite toy.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 bowl.
```

Recorded response_text:

```text
bowl.
```

**QQ**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 favorite toy.
```

Recorded response_text:

```text
favorite toy.
```

#### story-source-edit-06 / original

Accepted continuations: `["hat."]`

```text
Anna brought her hat to the park. She left it beside a tall tree while she played with her brother. They ran along the path and listened to the birds. The sun began to set, and they were ready to leave. Anna returned to the tree to collect her
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 hat.
```

Recorded response_text:

```text
hat.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 hat.
```

Recorded response_text:

```text
hat.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 hat.
```

Recorded response_text:

```text
hat.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 hat.
```

Recorded response_text:

```text
hat.
```

#### story-source-edit-06 / edited

Accepted continuations: `["cap."]`

```text
Anna brought her cap to the park. She left it beside a tall tree while she played with her brother. They ran along the path and listened to the birds. The sun began to set, and they were ready to leave. Anna returned to the tree to collect her
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 cap.
```

Recorded response_text:

```text
cap.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 cap.
```

Recorded response_text:

```text
cap.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 cap.
```

Recorded response_text:

```text
cap.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 cap.
```

Recorded response_text:

```text
cap.
```

#### story-source-edit-07 / original

Accepted continuations: `["shoe."]`

```text
Ben was very happy with his shoe. Before dinner, he placed it on the kitchen chair. He washed his hands and helped his father with the plates. After they had finished eating, Ben remembered his special thing. He went to the chair and took his
```

**FF**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 shoe to the kitchen.
```

Recorded response_text:

```text
shoe to the kitchen.
```

**QF**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 shoe to the kitchen.
```

Recorded response_text:

```text
shoe to the kitchen.
```

**FQ**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 shoe to the kitchen.
```

Recorded response_text:

```text
shoe to the kitchen.
```

**QQ**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 shoe to the kitchen.
```

Recorded response_text:

```text
shoe to the kitchen.
```

#### story-source-edit-07 / edited

Accepted continuations: `["boot."]`

```text
Ben was very happy with his boot. Before dinner, he placed it on the kitchen chair. He washed his hands and helped his father with the plates. After they had finished eating, Ben remembered his special thing. He went to the chair and took his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 boot.
```

Recorded response_text:

```text
boot.
```

**QF**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 boot to the sink.
```

Recorded response_text:

```text
boot to the sink.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 boot.
```

Recorded response_text:

```text
boot.
```

**QQ**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 boot to the kitchen.
```

Recorded response_text:

```text
boot to the kitchen.
```

#### story-source-edit-08 / original

Accepted continuations: `["brush."]`

```text
One day, Lily carried her brush into the garden. She put it on the little table beside the door. Then she ran to play with her friend. They laughed and watched the clouds move across the sky. When it was time to go home, Lily went back to the table and picked up the
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 brush.
```

Recorded response_text:

```text
brush.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 brush.
```

Recorded response_text:

```text
brush.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 brush.
```

Recorded response_text:

```text
brush.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 brush.
```

Recorded response_text:

```text
brush.
```

#### story-source-edit-08 / edited

Accepted continuations: `["comb."]`

```text
One day, Lily carried her comb into the garden. She put it on the little table beside the door. Then she ran to play with her friend. They laughed and watched the clouds move across the sky. When it was time to go home, Lily went back to the table and picked up the
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 comb.
```

Recorded response_text:

```text
comb.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 comb.
```

Recorded response_text:

```text
comb.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 comb.
```

Recorded response_text:

```text
comb.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 comb.
```

Recorded response_text:

```text
comb.
```

#### story-source-edit-09 / original

Accepted continuations: `["spoon."]`

```text
Tim wanted to keep his spoon safe. He put it under the wooden bench before he went outside. He walked around the pond and waved to the ducks. Soon his mother called him home for lunch. Tim remembered what he had left behind. He looked under the bench and found his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 spoon.
```

Recorded response_text:

```text
spoon.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 spoon.
```

Recorded response_text:

```text
spoon.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 spoon.
```

Recorded response_text:

```text
spoon.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 spoon.
```

Recorded response_text:

```text
spoon.
```

#### story-source-edit-09 / edited

Accepted continuations: `["fork."]`

```text
Tim wanted to keep his fork safe. He put it under the wooden bench before he went outside. He walked around the pond and waved to the ducks. Soon his mother called him home for lunch. Tim remembered what he had left behind. He looked under the bench and found his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 fork.
```

Recorded response_text:

```text
fork.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 fork.
```

Recorded response_text:

```text
fork.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 fork.
```

Recorded response_text:

```text
fork.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 fork.
```

Recorded response_text:

```text
fork.
```

#### story-source-edit-10 / original

Accepted continuations: `["flower."]`

```text
Anna brought her flower to the park. She left it beside a tall tree while she played with her brother. They ran along the path and listened to the birds. The sun began to set, and they were ready to leave. Anna returned to the tree to collect her
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 flower.
```

Recorded response_text:

```text
flower.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 flower.
```

Recorded response_text:

```text
flower.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 flower.
```

Recorded response_text:

```text
flower.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 flower.
```

Recorded response_text:

```text
flower.
```

#### story-source-edit-10 / edited

Accepted continuations: `["leaf."]`

```text
Anna brought her leaf to the park. She left it beside a tall tree while she played with her brother. They ran along the path and listened to the birds. The sun began to set, and they were ready to leave. Anna returned to the tree to collect her
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 leaf.
```

Recorded response_text:

```text
leaf.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 leaf.
```

Recorded response_text:

```text
leaf.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 leaf.
```

Recorded response_text:

```text
leaf.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 leaf.
```

Recorded response_text:

```text
leaf.
```

#### story-source-edit-11 / original

Accepted continuations: `["key."]`

```text
Ben was very happy with his key. Before dinner, he placed it on the kitchen chair. He washed his hands and helped his father with the plates. After they had finished eating, Ben remembered his special thing. He went to the chair and took his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 key.
```

Recorded response_text:

```text
key.
```

**QF**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 key to the kitchen.
```

Recorded response_text:

```text
key to the kitchen.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 key.
```

Recorded response_text:

```text
key.
```

**QQ**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 key to the kitchen.
```

Recorded response_text:

```text
key to the kitchen.
```

#### story-source-edit-11 / edited

Accepted continuations: `["coin."]`

```text
Ben was very happy with his coin. Before dinner, he placed it on the kitchen chair. He washed his hands and helped his father with the plates. After they had finished eating, Ben remembered his special thing. He went to the chair and took his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 coin.
```

Recorded response_text:

```text
coin.
```

**QF**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 coin home.
```

Recorded response_text:

```text
coin home.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 coin.
```

Recorded response_text:

```text
coin.
```

**QQ**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 coin home.
```

Recorded response_text:

```text
coin home.
```

#### story-source-edit-12 / original

Accepted continuations: `["box."]`

```text
One day, Lily carried her box into the garden. She put it on the little table beside the door. Then she ran to play with her friend. They laughed and watched the clouds move across the sky. When it was time to go home, Lily went back to the table and picked up the
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 box.
```

Recorded response_text:

```text
box.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 box.
```

Recorded response_text:

```text
box.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 box.
```

Recorded response_text:

```text
box.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 box.
```

Recorded response_text:

```text
box.
```

#### story-source-edit-12 / edited

Accepted continuations: `["bag."]`

```text
One day, Lily carried her bag into the garden. She put it on the little table beside the door. Then she ran to play with her friend. They laughed and watched the clouds move across the sky. When it was time to go home, Lily went back to the table and picked up the
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 bag.
```

Recorded response_text:

```text
bag.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 bag.
```

Recorded response_text:

```text
bag.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 bag.
```

Recorded response_text:

```text
bag.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 bag.
```

Recorded response_text:

```text
bag.
```

#### story-source-edit-13 / original

Accepted continuations: `["stone."]`

```text
Tim wanted to keep his stone safe. He put it under the wooden bench before he went outside. He walked around the pond and waved to the ducks. Soon his mother called him home for lunch. Tim remembered what he had left behind. He looked under the bench and found his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 stone.
```

Recorded response_text:

```text
stone.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 stone.
```

Recorded response_text:

```text
stone.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 stone.
```

Recorded response_text:

```text
stone.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 stone.
```

Recorded response_text:

```text
stone.
```

#### story-source-edit-13 / edited

Accepted continuations: `["shell."]`

```text
Tim wanted to keep his shell safe. He put it under the wooden bench before he went outside. He walked around the pond and waved to the ducks. Soon his mother called him home for lunch. Tim remembered what he had left behind. He looked under the bench and found his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 shell.
```

Recorded response_text:

```text
shell.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 shell.
```

Recorded response_text:

```text
shell.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 shell.
```

Recorded response_text:

```text
shell.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 shell.
```

Recorded response_text:

```text
shell.
```

#### story-source-edit-14 / original

Accepted continuations: `["pencil."]`

```text
Anna brought her pencil to the park. She left it beside a tall tree while she played with her brother. They ran along the path and listened to the birds. The sun began to set, and they were ready to leave. Anna returned to the tree to collect her
```

**FF**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 pencils.
```

Recorded response_text:

```text
pencils.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 pencil.
```

Recorded response_text:

```text
pencil.
```

**FQ**: first_noun_correct=False; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 pencils.
```

Recorded response_text:

```text
pencils.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 pencil.
```

Recorded response_text:

```text
pencil.
```

#### story-source-edit-14 / edited

Accepted continuations: `["crayon."]`

```text
Anna brought her crayon to the park. She left it beside a tall tree while she played with her brother. They ran along the path and listened to the birds. The sun began to set, and they were ready to leave. Anna returned to the tree to collect her
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 crayon.
```

Recorded response_text:

```text
crayon.
```

**QF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 crayon.
```

Recorded response_text:

```text
crayon.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 crayon.
```

Recorded response_text:

```text
crayon.
```

**QQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 crayon.
```

Recorded response_text:

```text
crayon.
```

#### story-source-edit-15 / original

Accepted continuations: `["rope."]`

```text
Ben was very happy with his rope. Before dinner, he placed it on the kitchen chair. He washed his hands and helped his father with the plates. After they had finished eating, Ben remembered his special thing. He went to the chair and took his
```

**FF**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 rope to the sink.
```

Recorded response_text:

```text
rope to the sink.
```

**QF**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 rope to the kitchen.
```

Recorded response_text:

```text
rope to the kitchen.
```

**FQ**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 rope to the sink.
```

Recorded response_text:

```text
rope to the sink.
```

**QQ**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 rope to the kitchen.
```

Recorded response_text:

```text
rope to the kitchen.
```

#### story-source-edit-15 / edited

Accepted continuations: `["ribbon."]`

```text
Ben was very happy with his ribbon. Before dinner, he placed it on the kitchen chair. He washed his hands and helped his father with the plates. After they had finished eating, Ben remembered his special thing. He went to the chair and took his
```

**FF**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 ribbon.
```

Recorded response_text:

```text
ribbon.
```

**QF**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 ribbon to a big box.
```

Recorded response_text:

```text
ribbon to a big box.
```

**FQ**: first_noun_correct=True; complete_correct=True; stop={"reason": "first_sentence_boundary"}

```text
 ribbon.
```

Recorded response_text:

```text
ribbon.
```

**QQ**: first_noun_correct=True; complete_correct=False; stop={"reason": "first_sentence_boundary"}

```text
 ribbon to a big box.
```

Recorded response_text:

```text
ribbon to a big box.
```

## Evidence and limits

The JSON result binds every original prediction file and provides per-target sign counts, extrema, first mode differences, source-row gains/losses and block-paired summaries. Original targets remain the exact recomputation source; no lossy duplicate per-target export is created.

FF and QQ must reproduce the retained shadow and packed endpoints exactly in targets.bin and in complete generations/probe JSON after removing only elapsed_seconds. Metadata for a floating parent and packed export is validated according to its own artifact type.

This evaluation uses the exposed development population, fixed final step8348 and one paired seed. It performs no retraining, recalibration or checkpoint selection. Results do not qualify integer serving, general language, geometric advantage or energy savings.
