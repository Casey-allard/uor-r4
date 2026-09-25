# Learned integer-code choices: complete source and generation record

Previously exposed development; fixed final candidate. Principal qualitative acceptance is recorded separately.

| Arm | Continuous | Projected parent | Learned hard codes | NoRead | Hard minus continuous |
|---|---:|---:|---:|---:|---:|
| quaternion | 2.090518499 | 2.150319798 | 2.114226169 | 2.609608832 | +0.023707670 |
| householder_pair | 2.064403221 | 2.147928712 | 2.110879800 | 2.607989949 | +0.046476579 |

## quaternion

### Source counts

| Artifact | First noun /32 | Exact completion /32 | Complete pairs /16 |
|---|---:|---:|---:|
| rung1_parent | 28 | 28 | 14 |
| parent_ptq | 25 | 23 | 10 |
| continuous | 26 | 23 | 9 |
| unprojected | 27 | 19 | 8 |
| projected | 26 | 23 | 11 |
| rounding | 29 | 28 | 12 |

### Every source response

Text below is preserved as JSON string literals. Gains do not cancel losses.

#### story-source-edit-00 / original

Accepted: ["apple."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "apple."

parent_ptq: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "door."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "apple."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "apple."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "apple."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "apple."

#### story-source-edit-00 / edited

Accepted: ["pear."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "pear."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "pear."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "pear."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "pear."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "pear."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "pear."

#### story-source-edit-01 / original

Accepted: ["ball."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "ball."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "ball."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "ball."

unprojected: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "ball in the grass."

projected: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "ball stuck in the wooden."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "ball."

#### story-source-edit-01 / edited

Accepted: ["kite."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "kite."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "kite."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "kite."

unprojected: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "kite stuck in the wooden."

projected: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "kite stuck in the wooden."

rounding: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "kite stuck in the water."

#### story-source-edit-02 / original

Accepted: ["book."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "book."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "book."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "book."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "book."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "book."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "book."

#### story-source-edit-02 / edited

Accepted: ["toy."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "toy."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "toy."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "toy."

unprojected: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "toy car."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "toy."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "toy."

#### story-source-edit-03 / original

Accepted: ["bell."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bell."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bell."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bell."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bell."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bell."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bell."

#### story-source-edit-03 / edited

Accepted: ["drum."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "drum."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "drum."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "drum."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "drum."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "drum."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "drum."

#### story-source-edit-04 / original

Accepted: ["doll."]

rung1_parent: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "door."

parent_ptq: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "door."

continuous: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "door."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "doll."

projected: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "door."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "doll."

#### story-source-edit-04 / edited

Accepted: ["bear."]

rung1_parent: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "door."

parent_ptq: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "door."

continuous: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "door."

unprojected: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "door."

projected: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "door."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bear."

#### story-source-edit-05 / original

Accepted: ["cup."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "cup."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "cup."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "cup."

unprojected: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "cup in the water."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "cup."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "cup."

#### story-source-edit-05 / edited

Accepted: ["bowl."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bowl."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bowl."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bowl."

unprojected: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "bowl in the wooden."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bowl."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bowl."

#### story-source-edit-06 / original

Accepted: ["hat."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "hat."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "hat."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "hat."

unprojected: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "hat and make it go fast."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "hat."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "hat."

#### story-source-edit-06 / edited

Accepted: ["cap."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "cap."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "cap."

continuous: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "captain."

unprojected: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "captain."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "cap."

rounding: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "captain."

#### story-source-edit-07 / original

Accepted: ["shoe."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "shoe."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "shoe."

continuous: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "shoe home."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "shoe."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "shoe."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "shoe."

#### story-source-edit-07 / edited

Accepted: ["boot."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "boot."

parent_ptq: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "boot back."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "boot."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "boot."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "boot."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "boot."

#### story-source-edit-08 / original

Accepted: ["brush."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "brush."

parent_ptq: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "door."

continuous: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "door."

unprojected: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "door."

projected: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "door."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "brush."

#### story-source-edit-08 / edited

Accepted: ["comb."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "comb."

parent_ptq: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "door."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "comb."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "comb."

projected: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "door."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "comb."

#### story-source-edit-09 / original

Accepted: ["spoon."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "spoon."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "spoon."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "spoon."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "spoon."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "spoon."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "spoon."

#### story-source-edit-09 / edited

Accepted: ["fork."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "fork."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "fork."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "fork."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "fork."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "fork."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "fork."

#### story-source-edit-10 / original

Accepted: ["flower."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "flower."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "flower."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "flower."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "flower."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "flower."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "flower."

#### story-source-edit-10 / edited

Accepted: ["leaf."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "leaf."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "leaf."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "leaf."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "leaf."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "leaf."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "leaf."

#### story-source-edit-11 / original

Accepted: ["key."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "key."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "key."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "key."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "key."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "key."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "key."

#### story-source-edit-11 / edited

Accepted: ["coin."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "coin."

parent_ptq: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "coin home."

continuous: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "coin home."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "coin."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "coin."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "coin."

#### story-source-edit-12 / original

Accepted: ["box."]

rung1_parent: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "door."

parent_ptq: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "door."

continuous: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "door."

unprojected: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "door."

projected: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "door."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "box."

#### story-source-edit-12 / edited

Accepted: ["bag."]

rung1_parent: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "door."

parent_ptq: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "door."

continuous: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "door."

unprojected: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "door."

projected: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "door."

rounding: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "clouds."

#### story-source-edit-13 / original

Accepted: ["stone."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "stone."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "stone."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "stone."

unprojected: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "stone under the wooden back."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "stone."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "stone."

#### story-source-edit-13 / edited

Accepted: ["shell."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "shell."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "shell."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "shell."

unprojected: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "shell under the wooden back."

projected: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "shell under the wooden bench."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "shell."

#### story-source-edit-14 / original

Accepted: ["pencil."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "pencil."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "pencil."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "pencil."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "pencil."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "pencil."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "pencil."

#### story-source-edit-14 / edited

Accepted: ["crayon."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "crayon."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "crayon."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "crayon."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "crayon."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "crayon."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "crayon."

#### story-source-edit-15 / original

Accepted: ["rope."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "rope."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "rope."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "rope."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "rope."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "rope."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "rope."

#### story-source-edit-15 / edited

Accepted: ["ribbon."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "ribbon."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "ribbon."

continuous: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "ribbon to his father."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "ribbon."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "ribbon."

rounding: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "father."

### Actual free generations


#### rung1_parent: generation 0, seed 2014

Prompt: "Once upon a time, there was a mommy and daddy playing with their 3-year-old son. Mommy"

Stop: {"reason": "maximum_new_tokens"}

    "had a job to take a long nap under a big tree on the grass. The mommy told her, \"We must take a cloth from the tree to get it back.\"\nThe mommy said. \"Threeze when I want to get you so big?\" The mommy said, “No, mommy. I'm glad you like this nap!”\nThe mommy thanked Mommy and said, “I’m going to look at the job from the tree. You can keep it on!”\nThe mommy agreed, “I’m sorry, but let’s get an adventure. I'm sure you can use this nap after.”"

#### rung1_parent: generation 1, seed 2015

Prompt: "Once upon a time, there was a mighty lion walking through the jungle. He was looking for his friends, and he"

Stop: {"reason": "maximum_new_tokens"}

    "found his best friend, a small squirrel. The squirrel was very happy to be all sorting the fun finding the lions.\nOne day, the mighty lion met a big fish named Bob. They wanted to teach all the fun things they had made. The mighty lion was happy and went to find the fun things they were. They all felt happy and happy.\nThe mighty lion and Bob decided to go on the journey to start their journey to find the way out of the journey. They found a big bush and a big smile. They both smiled and played with the new things from the mighty lion. The mighty lion was happy"

#### rung1_parent: generation 2, seed 2016

Prompt: "Once there was a pink goose. She was very pretty. The goose liked to eat grass, corn and bugs. One"

Stop: {"reason": "eos"}

    "day, it was an invitation for his friend, Tom. \"Let's rest on the invitation to go!\" Tom said. They went to look for him. They found the best spots. Tom and Tom were very happy. They sat and talked all day long."

#### rung1_parent: generation 3, seed 2017

Prompt: "One day, a boy named Tim wanted to ride his bike. He loved to ride up the big hill near his house"

Stop: {"reason": "eos"}

    ". He had a great idea to help his friend.\nThe little girl and the boy played with the toy. They were so happy. They laughed and had fun.\nBut then, something unexpected happened. When they got tired, they sat down to rest. The little girl was sad because she did not have the toy to help him have lots of fun. The boy looked in the mirror and saw a tiny bird named Lily. Lily was playing and wanted to play with it. They both knew it was not fair. They all became friends and played together in the mirror."

#### rung1_parent: generation 4, seed 2018

Prompt: "One day, there was a small and modest train. Inside the train were two friends - Lily and Dave. They"

Stop: {"reason": "maximum_new_tokens"}

    "all lived in the train and they played together. They had lots of fun at the train and had so much fun.\nOne day, Lily and Dave went to the train and started to play. The train was very happy to see and play with them all day. The sun set on, and they played together. They had lots of fun.\nBut then, something unexpected happened. A big, friendly dog came to the train! The dog wanted to play too. But the dog was not nice at all. The dog said, \"I can't play with my toys. I can't have my toys.\" The train was"

#### parent_ptq: generation 0, seed 2014

Prompt: "Once upon a time, there was a mommy and daddy playing with their 3-year-old son. Mommy"

Stop: {"reason": "maximum_new_tokens"}

    "'s best thing was always so so after all the syookear-bon. She ran to the window and saw a fun feeling inside. It was a very special, and Mommy had with a piece of a carrot.\nJane asked, “Look what is my mommy and she could be very special?”\nJane said, “I what’s so always special.”\nJane and daddy nodded and headed out. “When I can get my best, mommy and Mommy replied, “Yes\".\nJane and daddy took her and jumped in the window. They put on their shoes and put on a blue carrot on it."

#### parent_ptq: generation 1, seed 2015

Prompt: "Once upon a time, there was a mighty lion walking through the jungle. He was looking for his friends, and he"

Stop: {"reason": "maximum_new_tokens"}

    "knew he could not find his friends, but he was too small to be hungry. \nOne day, a little boy named Tim came to visit. He saw a deep-lid boy named Tom. Tom did not want to play with Tim and his friends. They all played together and had fun.\nBut then, something unexpected happened. The lion started to cry. The lion said, \"I don't want to play near the jungle.\" The lion said, \"I can do it, Tim! Just be careful and help us get wet.\"\nThe lion and Tom were still very happy. They played together for the rest"

#### parent_ptq: generation 2, seed 2016

Prompt: "Once there was a pink goose. She was very pretty. The goose liked to eat grass, corn and bugs. One"

Stop: {"reason": "maximum_new_tokens"}

    "day, a little bird came to the goose's goose. The goose wanted to play with him.\nThe goose said, \"I can play play with you!\" The goose liked its idea. He wanted to help his friend. So, he gave his little goose to the little bird. The goose said, \"Thank you, goose, if you are not good at the goose. I love to eat you too.\"\nThe goose was happy. He wanted to eat the little bird. So, he went to the goose. The goose said, \"Thank you, goose! I will eat some of grass and flowers. I am very"

#### parent_ptq: generation 3, seed 2017

Prompt: "One day, a boy named Tim wanted to ride his bike. He loved to ride up the big hill near his house"

Stop: {"reason": "maximum_new_tokens"}

    ".\nOn the way, he saw a big cat. The cat said, \"Hi, cat! That's not nice to you. I don't want to be nice.\"\nThe cat was sad. Then, a girl named Sue came and saw a big cat. Tim was hungry and wanted to help.\nLily and Sue looked not big to each other. They both did not want to give up. They talked about how much fun it could. They all went to look for the nice cat and the cat.\nThe moral of the story is that when you help others, you can make friends safe from just playing in the"

#### parent_ptq: generation 4, seed 2018

Prompt: "One day, there was a small and modest train. Inside the train were two friends - Lily and Dave. They"

Stop: {"reason": "maximum_new_tokens"}

    "would laugh at each other and each other each other ass they would play catch.\nOne day, Inside the train was playing in the sky, but it was too small and had to be sad. So, she said, \"I will be sad if he wants to be small with this train.\"\nSo, they went to find a way to find a train to be small.\nAs they went, they saw a big and bright train that had many of them.\nThe train was so happy. She said, \"Thank you, train, for helping me be my train.\"\nThe train was so happy that she could"

#### continuous: generation 0, seed 2014

Prompt: "Once upon a time, there was a mommy and daddy playing with their 3-year-old son. Mommy"

Stop: {"reason": "maximum_new_tokens"}

    "loved to mix things from the cupboard. One day, when we mix the things from the cupboard, they were not supposed they had all the time.\nOn the way, they found to mix it! The two friends had to put on it so it could be very popular in the cupboard.\nThe mommy was so happy to have brought out a special sigh. \nAt the end of the day the cupboard was to go home. \nThe mommy was not upset, and the 3 year-old was very excited. The mommy said she could use the cupboard to make a big cake with the cupboard! The 3"

#### continuous: generation 1, seed 2015

Prompt: "Once upon a time, there was a mighty lion walking through the jungle. He was looking for his friends, and he"

Stop: {"reason": "maximum_new_tokens"}

    "knew he needed to find a special adventure. He searched and searched, but he couldn't find his adventure. He saw a big, red ball inside.\nHe ran towards the ball, but he did not knowly. He just wanted to make anyway. He saw a big tree with a hole in it. It was a tall, yellow ball. He was so excited!\nHe asked the ball if he could put it for his friends. He said, \"Yes, it would be a fun surprise.\"\nThe lion was curious. He asked his friends to help the ball be there.\nThe lion was not a good"

#### continuous: generation 2, seed 2016

Prompt: "Once there was a pink goose. She was very pretty. The goose liked to eat grass, corn and bugs. One"

Stop: {"reason": "eos"}

    "day, She saw a big tree. She was curious. She opened the tree and took some water. The goose was so happy to have a eat cake.\nThe goose flew into the tree. The goose looked very pretty and pretty. It became his best friend, the sun. They played together all day long. And the goose was pretty and happy."

#### continuous: generation 3, seed 2017

Prompt: "One day, a boy named Tim wanted to ride his bike. He loved to ride up the big hill near his house"

Stop: {"reason": "eos"}

    ". Tim saw a big, big red ball in his yard. The ball was very soft and had many legs.\nTim called his friend, Sue. \"Look, Sue! I found a ball!\" Sue asked. Sue was surprised and said, \"Wow, that's just a big red ball.\"\nTim and Sue played with the fun dogs. They were very happy together. The big red ball was their best day."

#### continuous: generation 4, seed 2018

Prompt: "One day, there was a small and modest train. Inside the train were two friends - Lily and Dave. They"

Stop: {"reason": "eos"}

    "all played games and had fun.\nOne day, Lily said to Dave, \"We can buy toys for a long time ago!\" said the two friends were happy and they had a great time together. They had a great time at the train and became good friends.\nAfter they were both the two friends, they were the best of friends. They all loved to play together and play with the train. And they always never look at each other."

#### unprojected: generation 0, seed 2014

Prompt: "Once upon a time, there was a mommy and daddy playing with their 3-year-old son. Mommy"

Stop: {"reason": "maximum_new_tokens"}

    "loved to take a photo by the fridge. \nLeith with excitement and his mom said, \"Ok, so let me go on a photo!\" \nThe mommy were very pleased.\n\"That is that, she does,\" said Mum. \nThe mommy smiled and said, \"That sounds fun!\" Mommy said, \"Well done, Mommy! I'll show you to the fridge.\"  \nLeith started to phade of Mommy. He asked ‘Out you know if she could fly!\" \nLeith was very sad.\nBut then she noticed a"

#### unprojected: generation 1, seed 2015

Prompt: "Once upon a time, there was a mighty lion walking through the jungle. He was looking for his friends, and he"

Stop: {"reason": "eos"}

    "would run to the jungle. The mighty lion was very happy.\nOne day, a little bird flew out of the jungle. The bird was singing a beautiful song. The little bird saw the bird singing and said, \"Hello, little bird! Where are you?\" The bird was so happy that he could not reach the bird's song.\nThe little bird tried to help the bird's song, but the bird did not mind. The mighty lion tried to mind the bird, and it was too hard for the bird. The lion learned to be more careful and not touch the mighty lion."

#### unprojected: generation 2, seed 2016

Prompt: "Once there was a pink goose. She was very pretty. The goose liked to eat grass, corn and bugs. One"

Stop: {"reason": "maximum_new_tokens"}

    "day, her mommy came into the kitchen. She saw the goose in a corner and knew her mommy was right.\nShe started to cry. \"But no goose,\" her mommy said. Her mommy said, \"It's okay, we can make her mommy happy. The sun is hot and warm. Do you want to eat it?\" \nThe goose thought it was a good idea and decided to share the goose with her mommy. She took it out and put it back in the pink corn. She was so happy and thanked her mommy. \nThe goose was never seen again! After a few minutes, the goose tried to"

#### unprojected: generation 3, seed 2017

Prompt: "One day, a boy named Tim wanted to ride his bike. He loved to ride up the big hill near his house"

Stop: {"reason": "maximum_new_tokens"}

    ". Tim's mom was happy. Tim wanted to see the bike too.\nTim said to his friend, a little dog named Max. Max saw the bike and said, \"Hi, Tim! I want a bike too.\" Max looked at the bike and said, \"Hello, bike! Let's ride it together!\" They looked and ran and played together.\nAs Tim tried to ride the bike, Tim was faster. He was scared of Max. He did not want to ride the bike away. Tim felt sad and wanted to help his friend.\nTim tried to ride the bike, but he knew he needed to rest."

#### unprojected: generation 4, seed 2018

Prompt: "One day, there was a small and modest train. Inside the train were two friends - Lily and Dave. They"

Stop: {"reason": "maximum_new_tokens"}

    "wanted to talk to the train and take it home and show them to his mom.\nOne day, when Lily was about to eat the train, Lily noticed a big pile of the train. She was so excited! She ran to the train and said, \"Look at my train!\" said the train was so happy that it made the train go around and started to get the train.\nBut Lily didn't listen. She was afraid she would be punished. She asked the train, \"Why are you so sad?\"\nThe train said, \"You can't get train. That's not your train! It was your train"

#### projected: generation 0, seed 2014

Prompt: "Once upon a time, there was a mommy and daddy playing with their 3-year-old son. Mommy"

Stop: {"reason": "maximum_new_tokens"}

    "gave the 3 year old and went outside to play. earury asked Mommy, “You can have lots of fun!” His mother said, “That’s a sweet treat! I set it.” \nSally was very excited. \nBut when Mommy arrived at the sight, Mommy noticed a big, funny clown! \nWhen Mommy said, “Let’s go home, Mommy!\"\nSally ran to the same corner and said, \"You can have of right, so don’t go inside. It's a great day.\" \nSally was so excited. She put the treat on her special rock and"

#### projected: generation 1, seed 2015

Prompt: "Once upon a time, there was a mighty lion walking through the jungle. He was looking for his friends, and he"

Stop: {"reason": "eos"}

    "went to his friend, a small squirrel named Sam. Sam was very mighty.\nHe said, \"If I climb this mighty lion, I will give you a big squirrel.\" Sam barked and wagged his tail. After the squirrel played, the lion noticed a big, friendly squirrel. Sam and Sam were so happy!\nBob and Sam played a game where they could run and jump and splash in the puddles. Sam was very proud of himself and they played with his big, friendly one. The mighty lion."

#### projected: generation 2, seed 2016

Prompt: "Once there was a pink goose. She was very pretty. The goose liked to eat grass, corn and bugs. One"

Stop: {"reason": "maximum_new_tokens"}

    "day, her friend Sam came to the park. \"Hi, I am pink! Let's eat it for dinner,\" her friend said.\nHonry Sam's little girl went to the park. He said to Sam, \"I can eat grass for grass. Let's eat grass and eat.\"\nHonry smiled and said \"Thank you, goose. You are very kind to me.\"\nThe goose was very happy and said \"You can share the grass for everyone.\" The pink goose smiled and said \"I like to eat it too!\"\nThe goose learned that being pink became pink and made her happy.\nThe"

#### projected: generation 3, seed 2017

Prompt: "One day, a boy named Tim wanted to ride his bike. He loved to ride up the big hill near his house"

Stop: {"reason": "eos"}

    ". Tim saw a big ball. The ball was to roll around, but Tim was sad.\nTim found a long stick near the big hill. He went to the ball and saw a big red ball. But the ball was too big for him to reach it. Tim got scared and tried to escape, but it was too high for any hill.\nTim saw a tiny box. The ball was not open. Tim was sad. He said, \"Don't worry, little box. But maybe for help, then we can reach the ball.\" Tim agreed, and they played together all day."

#### projected: generation 4, seed 2018

Prompt: "One day, there was a small and modest train. Inside the train were two friends - Lily and Dave. They"

Stop: {"reason": "maximum_new_tokens"}

    "both had two best friends.\nOne day, a little boy came to Dave's birthday. Lily came over and showed it to Dave. She said \"I don't know, Dave. It is my best friend, Dave!\"\n\"Come on, Dave, why are you so excited?\" Dave asked the boy.\n\"Yes, Dave, I want to fly,\" said Lily. \"You're on the train! I'm so happy to see Dave\".\nRave was so excited!\nWhen Dave went to the train with Dave, he saw a big pile of crayons."

#### rounding: generation 0, seed 2014

Prompt: "Once upon a time, there was a mommy and daddy playing with their 3-year-old son. Mommy"

Stop: {"reason": "maximum_new_tokens"}

    ", they were very happy and they wanted to play with them first.\n\"Mommy, it's time to play outside,\" Mommy said, with a superherossday, so they raced to get some of them.\nAt the end of the day, Mommy said goodbye and ran back to the house. Mommy said she had to promise to be careful to keep the end of the day they play.\nThen Mommy said back.\nAt the house, Mommy and Daddy opened the window. Her eyes lit up, and she was excited to see how it had been so much fun.\nWhen Daddy and Daddy finished their dinner"

#### rounding: generation 1, seed 2015

Prompt: "Once upon a time, there was a mighty lion walking through the jungle. He was looking for his friends, and he"

Stop: {"reason": "maximum_new_tokens"}

    "heard a voice. It was a dog named Max. Max was a big cat named Bob. Max wanted to catch him, but he was too strong.\nMax's owner, Tim saw the mighty lion and said he saw him. He said, \"I will catch you, mighty lion. You can't catch me, Max. It is all fun.\" Tim and Tim were very happy. They went back to Max's house and brought him back home.\nThe dog was so happy for Tim and Tim. Tim was happy too. Now, the mighty lion was not mighty enough to share and be nice. The mighty lion was happy"

#### rounding: generation 2, seed 2016

Prompt: "Once there was a pink goose. She was very pretty. The goose liked to eat grass, corn and bugs. One"

Stop: {"reason": "eos"}

    "day, he found a big, green bug on the ground. It was the same food he had ever seen.\nHe was so excited to look all around. He wanted to touch it and jump. He reached his friend, the cat, and started to get to the green bug. The cat looked at the bug and said, \"We can eat it, but I want to touch the bug too.\"\nThe green bug was very happy. He put the bug back to the green bug, and the green bug was a happy family. The bug felt happy too."

#### rounding: generation 3, seed 2017

Prompt: "One day, a boy named Tim wanted to ride his bike. He loved to ride up the big hill near his house"

Stop: {"reason": "eos"}

    ". Tim had a toy box of a big, happy dog.\nAt the big park, Tim saw his friend, Sue. Sue said, \"Hi, Tim! I can't ride my bike. Can we ride the box?\" Tim said, \"Yes, we can find the bike.\"\nAs they played together, they saw a fun ride on the beach. Tim took off his bike and ran to his friend. They played with the toy together. They had lots of fun. Tim learned that even when things seem scary, they can do anything. And they had a great time playing with the toy all day."

#### rounding: generation 4, seed 2018

Prompt: "One day, there was a small and modest train. Inside the train were two friends - Lily and Dave. They"

Stop: {"reason": "maximum_new_tokens"}

    "played all day long. One day, they found a big, red ball and the train's headst tail.\nThe train driver said, \"Welcome, train, train. It is very big and shiny.\" The train driver said, \"I know, so it looks red. Then I can show you how to make a happy face.\"\nThe train driver said, \"I think we can find the train.\" They sat next to the ball and said, \"Welcome, train!\"The train driver said, \"OK, train, go!\"\nThey worked together and found the train's head in the pond."

## householder_pair

### Source counts

| Artifact | First noun /32 | Exact completion /32 | Complete pairs /16 |
|---|---:|---:|---:|
| rung1_parent | 31 | 23 | 10 |
| parent_ptq | 31 | 22 | 10 |
| continuous | 32 | 26 | 12 |
| unprojected | 32 | 25 | 12 |
| projected | 31 | 22 | 10 |
| rounding | 30 | 24 | 10 |

### Every source response

Text below is preserved as JSON string literals. Gains do not cancel losses.

#### story-source-edit-00 / original

Accepted: ["apple."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "apple."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "apple."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "apple."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "apple."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "apple."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "apple."

#### story-source-edit-00 / edited

Accepted: ["pear."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "pear."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "pear."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "pear."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "pear."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "pear."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "pear."

#### story-source-edit-01 / original

Accepted: ["ball."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "ball."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "ball."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "ball."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "ball."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "ball."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "ball."

#### story-source-edit-01 / edited

Accepted: ["kite."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "kite."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "kite."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "kite."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "kite."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "kite."

rounding: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "kite, but he was still there."

#### story-source-edit-02 / original

Accepted: ["book."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "book."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "book."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "book."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "book."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "book."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "book."

#### story-source-edit-02 / edited

Accepted: ["toy."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "toy."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "toy."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "toy."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "toy."

projected: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "toy to play with her toy."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "toy."

#### story-source-edit-03 / original

Accepted: ["bell."]

rung1_parent: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "bell to the kitchen."

parent_ptq: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "bell to his bell."

continuous: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "bell to the kitchen."

unprojected: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "bell to the kitchen."

projected: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "bell to the kitchen."

rounding: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "bell to the kitchen."

#### story-source-edit-03 / edited

Accepted: ["drum."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "drum."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "drum."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "drum."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "drum."

projected: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "drum to the kitchen."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "drum."

#### story-source-edit-04 / original

Accepted: ["doll."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "doll."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "doll."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "doll."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "doll."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "doll."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "doll."

#### story-source-edit-04 / edited

Accepted: ["bear."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bear."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bear."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bear."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bear."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bear."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bear."

#### story-source-edit-05 / original

Accepted: ["cup."]

rung1_parent: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "cup, but he was not happy."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "cup."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "cup."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "cup."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "cup."

rounding: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "cup, but he was still there."

#### story-source-edit-05 / edited

Accepted: ["bowl."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bowl."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bowl."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bowl."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bowl."

projected: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "favorite toy."

rounding: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "bowl, but he did not know."

#### story-source-edit-06 / original

Accepted: ["hat."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "hat."

parent_ptq: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "hat and a big smile on her face."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "hat."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "hat."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "hat."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "hat."

#### story-source-edit-06 / edited

Accepted: ["cap."]

rung1_parent: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "captain."

parent_ptq: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "captain, but she was too scared to go."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "cap."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "cap."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "cap."

rounding: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "capes."

#### story-source-edit-07 / original

Accepted: ["shoe."]

rung1_parent: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "shoe home."

parent_ptq: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "shoe back to his shoe."

continuous: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "shoe to the kitchen."

unprojected: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "shoe to the hospital."

projected: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "shoe to the kitchen."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "shoe."

#### story-source-edit-07 / edited

Accepted: ["boot."]

rung1_parent: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "boot back to his house."

parent_ptq: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "boot back to his boot."

continuous: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "boot to the kitchen."

unprojected: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "boot to the kitchen."

projected: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "boot to the kitchen."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "boot."

#### story-source-edit-08 / original

Accepted: ["brush."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "brush."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "brush."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "brush."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "brush."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "brush."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "brush."

#### story-source-edit-08 / edited

Accepted: ["comb."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "comb."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "comb."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "comb."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "comb."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "comb."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "comb."

#### story-source-edit-09 / original

Accepted: ["spoon."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "spoon."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "spoon."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "spoon."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "spoon."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "spoon."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "spoon."

#### story-source-edit-09 / edited

Accepted: ["fork."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "fork."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "fork."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "fork."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "fork."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "fork."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "fork."

#### story-source-edit-10 / original

Accepted: ["flower."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "flower."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "flower."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "flower."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "flower."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "flower."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "flower."

#### story-source-edit-10 / edited

Accepted: ["leaf."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "leaf."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "leaf."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "leaf."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "leaf."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "leaf."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "leaf."

#### story-source-edit-11 / original

Accepted: ["key."]

rung1_parent: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "key to the table."

parent_ptq: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "key to his key."

continuous: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "key to the kitchen."

unprojected: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "key to the doctor."

projected: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "key to the kitchen."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "key."

#### story-source-edit-11 / edited

Accepted: ["coin."]

rung1_parent: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "coin home."

parent_ptq: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "coin inside."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "coin."

unprojected: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "coin home."

projected: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "coin home."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "coin."

#### story-source-edit-12 / original

Accepted: ["box."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "box."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "box."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "box."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "box."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "box."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "box."

#### story-source-edit-12 / edited

Accepted: ["bag."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bag."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bag."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bag."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bag."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bag."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "bag."

#### story-source-edit-13 / original

Accepted: ["stone."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "stone."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "stone."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "stone."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "stone."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "stone."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "stone."

#### story-source-edit-13 / edited

Accepted: ["shell."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "shell."

parent_ptq: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "shell under the wood."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "shell."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "shell."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "shell."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "shell."

#### story-source-edit-14 / original

Accepted: ["pencil."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "pencil."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "pencil."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "pencil."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "pencil."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "pencil."

rounding: first noun=False, exact=False, stop={"reason": "first_sentence_boundary"}

    "pencils."

#### story-source-edit-14 / edited

Accepted: ["crayon."]

rung1_parent: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "crayon."

parent_ptq: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "crayon."

continuous: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "crayon."

unprojected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "crayon."

projected: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "crayon."

rounding: first noun=True, exact=True, stop={"reason": "first_sentence_boundary"}

    "crayon."

#### story-source-edit-15 / original

Accepted: ["rope."]

rung1_parent: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "rope home."

parent_ptq: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "rope to the chair."

continuous: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "rope to the kitchen."

unprojected: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "rope to the park."

projected: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "rope to the kitchen."

rounding: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "rope to the kitchen."

#### story-source-edit-15 / edited

Accepted: ["ribbon."]

rung1_parent: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "ribbon to the chair."

parent_ptq: first noun=True, exact=False, stop={"reason": "short_cycle", "period": 2}

    "ribbon back to his father's father's father's"

continuous: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "ribbon to the kitchen."

unprojected: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "ribbon to a big house."

projected: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "ribbon to a big box."

rounding: first noun=True, exact=False, stop={"reason": "first_sentence_boundary"}

    "ribbon to his house."

### Actual free generations


#### rung1_parent: generation 0, seed 2014

Prompt: "Once upon a time, there was a mommy and daddy playing with their 3-year-old son. Mommy"

Stop: {"reason": "maximum_new_tokens"}

    "said, “Let’s have a plan to make it shiny!”\nDad looked at his mother and said, “I’re able to make the shinyest thing!\" \nThe mommy grabbed the paper and started to cut the shiny thing. After the cut, it started to start. It was so much fun! But when he got over, he was excited and said, “Yummy! I’m glad you are both beautiful!” \nDaddy smiled and replied, “You’ll be very careful.”\nDaddy told the mommy that he had to be careful and not like the cut. The shiny thing was"

#### rung1_parent: generation 1, seed 2015

Prompt: "Once upon a time, there was a mighty lion walking through the jungle. He was looking for his friends, and he"

Stop: {"reason": "maximum_new_tokens"}

    "couldn't believe his eyes. He couldn't wait to see what was inside the lion and he could never know.\nHe saw a little girl looking at the lion and told him to leave. The girl said, “What are you doing? That can make you sad and I have been punished. I need to punish you for being very kind and kind. You can have to be a little bit if you have me, no one can break.\"\nThe little girl was scared that she could not open her eyes. She thought to find someone who was there to help her. Suddenly, she heard a voice say, “I"

#### rung1_parent: generation 2, seed 2016

Prompt: "Once there was a pink goose. She was very pretty. The goose liked to eat grass, corn and bugs. One"

Stop: {"reason": "maximum_new_tokens"}

    "day, She decided to go to the park.\nThe goose walked around to find more bugs. He went to the corn and saw a big ball on the side. The ball looked sad.\nThe goose tried to help the ball. He thought for a friend to help the ball. The ball went too fast. It started to fall back. The ball was very sad. The goose wished for a friend to help him.\nThe goose went to the park. The big ball was still scared. It could not find any way. The goose was not happy. The pink goose was not find the ball. It was a sad ending"

#### rung1_parent: generation 3, seed 2017

Prompt: "One day, a boy named Tim wanted to ride his bike. He loved to ride up the big hill near his house"

Stop: {"reason": "eos"}

    ". Tim had a very good job. He had many friends in the hill.\nAt the hill, he saw a big fish. The fish said, \"Hi, Tim! I want to ride too. The fish is fun!\" Tim said, \"Yes, I want to ride with you.\" So, Tim and the fish ride all too, and they both had fun and the big fish was happy."

#### rung1_parent: generation 4, seed 2018

Prompt: "One day, there was a small and modest train. Inside the train were two friends - Lily and Dave. They"

Stop: {"reason": "maximum_new_tokens"}

    "wanted to introduce their names and everyone were so happy.\nMave was very excited when his friends came and ran over to the train. They were so excited and started to play. They had lots of fun and decided to play a game. Suddenly, a big box popped out of the train and the train started to play.\n\"Oh no, Dave!\" said Lily. \"I'm so happy!\" \nMave looked at the train and said, \"I'm sorry because I was so modest. You should have listened to me and the other people who were happy.\" \nMave was so"

#### parent_ptq: generation 0, seed 2014

Prompt: "Once upon a time, there was a mommy and daddy playing with their 3-year-old son. Mommy"

Stop: {"reason": "maximum_new_tokens"}

    "said, “Let’s have a bit lemon and make bubbles!\"\nBut daddy said “No, you have to take the other rush into a bit of lemon. If you try it can be “That’s important for this.\" \nJoe didn’t know why because he was just so he could use his lemon and bubbles so he could have found her bubbles. \nJoe was so disappointed that daddy was right in her. He was worried that he was scared he would help his he know if he was so lemon. His daddy was so happy that he was able to take the ron to his. He was"

#### parent_ptq: generation 1, seed 2015

Prompt: "Once upon a time, there was a mighty lion walking through the jungle. He was looking for his friends, and he"

Stop: {"reason": "maximum_new_tokens"}

    "couldn't find his way home. \nThe lion was so scared he decided he quickly and asked her dad to find his way home. \nThe lion was too nervous, but he was still able to find any friends. \nThen he thought of a plan. He hopped over and grabbed his friends in the big field.\nThe lion was so happy he was able to find his friends. He followed the lion to find more. He grabbed his friends and swam around, looking at the sky. He found a big tree and began, all the lions, and it was time to find out. \nThen, something"

#### parent_ptq: generation 2, seed 2016

Prompt: "Once there was a pink goose. She was very pretty. The goose liked to eat grass, corn and bugs. One"

Stop: {"reason": "eos"}

    "day, a little boy came to the goose. The boy said, \"Hello, what do you have in the goose? Why are you doing grass that is your grass? Why do you make with you?\" The boy did not know that his grass was not real. He asked his friend, a bird, for help.\nThe bird tried to help, but the goose was too small. The goose was sad. The bird did not know that it was just a real bird. The boy became friends and played together every day."

#### parent_ptq: generation 3, seed 2017

Prompt: "One day, a boy named Tim wanted to ride his bike. He loved to ride up the big hill near his house"

Stop: {"reason": "eos"}

    ". His favorite bike was in a big box. He was very happy because he loved the big hill.\nTim went to the big hill and he found many friends. They all had fun and played together every day. Tim and the friends were happy. They had so much fun that they had each other's bike."

#### parent_ptq: generation 4, seed 2018

Prompt: "One day, there was a small and modest train. Inside the train were two friends - Lily and Dave. They"

Stop: {"reason": "eos"}

    "liked to play and have fun together. One day, they decided to find some food. The train and the friends were scared, but they did not know what to do.\nThe train said, \"I am looking for food, but the train was not so modest. The train could not move. It said, \"Please, friends are you in the train. I will help you.\"\nThe train and Dave felt sad because of the train. They thought about it and decided to help them. In the end, the train was not modest anymore. The train and Lily became friends with the train and had fun together."

#### continuous: generation 0, seed 2014

Prompt: "Once upon a time, there was a mommy and daddy playing with their 3-year-old son. Mommy"

Stop: {"reason": "maximum_new_tokens"}

    "and daddy lived in a big house. \nOne day in a dark, she found a shiny thing in the park. It was an onion! The onion was so big that it started to put on fire. \nThe onion said, \"We should come to your house!\" it was very hot. \nMary's mommy and dad said, \"Welcome, mommy! We'll be happy to try!\"  \nSo, the mommy and dad took fires and hot onion bottles. \nMary was happy to have found the onion. She was so happy to have spent the day."

#### continuous: generation 1, seed 2015

Prompt: "Once upon a time, there was a mighty lion walking through the jungle. He was looking for his friends, and he"

Stop: {"reason": "eos"}

    "couldn't see the most beautiful land on the ground.\nHe asked his friend, Sally the bear, Sally asked him. \"What are you doing?\" Sally replied.\n\"It is a mighty land!\" Sally explained that the lion was the owner who was also for a good time. \nThe lion started to smile and said \"I have a good idea!\" \nSo, the lion was happy that Sally was safe and played with his mighty land. He still knew that Sally asked her friends to be nice and be kind to other people's friends."

#### continuous: generation 2, seed 2016

Prompt: "Once there was a pink goose. She was very pretty. The goose liked to eat grass, corn and bugs. One"

Stop: {"reason": "eos"}

    "day, while walking around the goose, the goose saw a big cloud. The cloud had bright eyes and the goose wanted to play.\nThe cloud took the big cloud to its home. It had a big net with its beak. The cloud was scared, but the cloud's net came. The goose tried to make a new friend, just a cloud.\nThe cloud saw the cloud was not scared anymore. The cloud flew up and down the goose's net. The big cloud was so happy that the cloud helped the cloud. The goose was happy too. The cloud flew away, and It was not scared anymore."

#### continuous: generation 3, seed 2017

Prompt: "One day, a boy named Tim wanted to ride his bike. He loved to ride up the big hill near his house"

Stop: {"reason": "maximum_new_tokens"}

    ". He had a big dog, a dog, and a little duck. They played together all day.\nTim saw a little girl named Sue. Sue had a big smile on her face. Sue said, \"Hi, I am your bike! Can I ride your bike up?\" Tim looked at Tim and said, \"No, you two silly. You are too big or very dirty.\"\nTim and Sue started to ride the bike together. They had so much fun. When it was time for dinner, Tim said, \"Thank you, Tim. You are very dirty.\" Tim smiled and said, \"You're welcome, Tim"

#### continuous: generation 4, seed 2018

Prompt: "One day, there was a small and modest train. Inside the train were two friends - Lily and Dave. They"

Stop: {"reason": "eos"}

    "loved to eat food and eat food and food. \nOne day, They decided to separate from them. They found a big, soft blanket. The train was very big and the train was very modest. They could not eat the train anymore.\nAs they were fighting, a little girl came to help. She was very good at separateing the train. She said, \"Thank you, little girl, for separated separate all.\" The train was happy that they worked together. From that day on, Lily and Dave always shared their toys and had many fun together."

#### unprojected: generation 0, seed 2014

Prompt: "Once upon a time, there was a mommy and daddy playing with their 3-year-old son. Mommy"

Stop: {"reason": "short_cycle", "period": 2}

    "and daddy had a lot of toys and daddy. They had much many toys, and they did not like to be rude.\n\"We are sorry, mommy. We can be rude and want this,\" their daddy said.\n\"Can we share his daddy's daddy?\" their mommy said.\n\"No, daddy. We don't want his daddy's daddy or's daddy's daddy's daddy"

#### unprojected: generation 1, seed 2015

Prompt: "Once upon a time, there was a mighty lion walking through the jungle. He was looking for his friends, and he"

Stop: {"reason": "eos"}

    "walked up a big hill. He felt a little scared, but he knew he was coming from the jungle.\nHe stepped on a big rock. He tried to run out of the jungle, but it was too a tiny. The lion looked for helter.\nHe stepped into the jungle, and he was free. He chased the lion, but he was too scared. He stepped onto the rock.\nThe lion looked forward to a beautiful view. He explored the jungle doors and had a long time. He was in awe. He was able to leave out of the jungle."

#### unprojected: generation 2, seed 2016

Prompt: "Once there was a pink goose. She was very pretty. The goose liked to eat grass, corn and bugs. One"

Stop: {"reason": "eos"}

    "day, She went for a walk with her friend, a big bear named Lily. Lily said, \"Let's go outside and play!\"\nLeo heard a small hill with the corn. He looked under it and behind the door. \"Can we have some?\"Leo said. Lily nodded and they climbed the hill. It was very tall.\nAfter a while, Lily and Lily were safe. They became best friends. They played and laughed together.\nThe moral of the story is to always help your friends and not to eat."

#### unprojected: generation 3, seed 2017

Prompt: "One day, a boy named Tim wanted to ride his bike. He loved to ride up the big hill near his house"

Stop: {"reason": "maximum_new_tokens"}

    ". Tim saw a big red car on the floor. He was sad because he wanted to play with the red car.\nTim asked his friend, a bird named Tim, for help. Tim said, \"Be careful, Tim. It is not safe. It is not fun.\" Tim was sad because he could not ride his bike. At Tim, Tim could not ride out his bike.\nTim went up to his house and said, \"What is wrong, Tim?\" Tim said, \"It's not a good bike. It is a big red car. Do you want to ride it too?\" The big red car did not"

#### unprojected: generation 4, seed 2018

Prompt: "One day, there was a small and modest train. Inside the train were two friends - Lily and Dave. They"

Stop: {"reason": "eos"}

    "all lived together in the train. Lily and Dave always listened to her train and they had a best.\nOne day, they found a big hole in the train. They had to go home and find their home.\nAs they were playing, they looked at the train.\nThe train said, \"I like your home. I am in a big, we can fly high.\"\nThe train was very happy and said, \"Thank you, train!\" They played with the train all day, even when it was time for a nap. The train was happy to find its home and Dave."

#### projected: generation 0, seed 2014

Prompt: "Once upon a time, there was a mommy and daddy playing with their 3-year-old son. Mommy"

Stop: {"reason": "eos"}

    "and daddy had a very special day."

#### projected: generation 1, seed 2015

Prompt: "Once upon a time, there was a mighty lion walking through the jungle. He was looking for his friends, and he"

Stop: {"reason": "eos"}

    "needed to take off the jungle. The lion was very sad and he did not like that his friends, so he saw his friends and they were never able to play again soon.\nThe lion was his friends and were always happy to help. The jungle was his friends and everyone was very happy. The lion was not sad anymore, and he lived happily ever after."

#### projected: generation 2, seed 2016

Prompt: "Once there was a pink goose. She was very pretty. The goose liked to eat grass, corn and bugs. One"

Stop: {"reason": "maximum_new_tokens"}

    "day, it was hot and cold. The goose wanted to go outside and play with their goose.\nWhile the goose went outside, the goose noticed something strange. It was not real! There were many trees, so many birds, corners, and even big. They tried to get the birds, but the goose was too big. So, the goose decided to eat the birds. The goose was very happy and could not wait to play with them all.\nThe next day, the goose went outside. It was too big for the goose to eat. The goose did not like the dog. The goose was sad and hungry."

#### projected: generation 3, seed 2017

Prompt: "One day, a boy named Tim wanted to ride his bike. He loved to ride up the big hill near his house"

Stop: {"reason": "maximum_new_tokens"}

    ".\nAs Tim rode, the hill was dark. Tim saw his friend, Sue, came over to play. Tim said, \"Wow, I like your bike!\" Sue said, \"Wow! I like a bike too!\" They played together, and Tim was very happy, and they played together with their bike.\nBut then, we like to go back to the beach and have fun!\" Tim said. They ran and jumped in the sun, but then they heard a voice. \"Quick, let's go!\" said Tim. Sue agreed and they started to play. They ran and ran and had lots of fun."

#### projected: generation 4, seed 2018

Prompt: "One day, there was a small and modest train. Inside the train were two friends - Lily and Dave. They"

Stop: {"reason": "maximum_new_tokens"}

    "played in a big house. They wanted to see their friends and family. Lily and Dave were happy in their house.\nIn the end, Lily and Dave went to the park. They saw a big hill. The train was very tall and could lift. They went on the swings and the train. The train was very strong.\nOne day, Dave found a new friend, a bird named Tim. Tim was too excited to help the bird. He asked the bird, \"Can I do you want to play with you?\" The bird said, \"Yes, you can play with me if you help me lift the"

#### rounding: generation 0, seed 2014

Prompt: "Once upon a time, there was a mommy and daddy playing with their 3-year-old son. Mommy"

Stop: {"reason": "maximum_new_tokens"}

    "had a special surprise. She was a kind boy, so little one. \nOne day, Mommy said they needed to buy new presents for the birthday party. She had a surprise for \nInside the party party, everyone like everyone in town. The party had a fun time, and everyone loved their birthday. Everyone clapped and danced when they played with every day. \nAfter a while, Mommy and Daddy went to an ancient room. There was a big, ugly bird. Mommy and their mommy were very happy. They had been very excited to play together. \nThe birthday party was in the party all day,"

#### rounding: generation 1, seed 2015

Prompt: "Once upon a time, there was a mighty lion walking through the jungle. He was looking for his friends, and he"

Stop: {"reason": "maximum_new_tokens"}

    "spotted a rabbit. The rabbit said hello, and he was very excited.\nHe grabbed his boat, Sam and the rabbit, and the rabbit went to the rabbit. \nSuddenly, the rabbit opened it and landed there. He was looking at them. He was amazed. He asked the rabbit, \"What are you doing?\" The rabbit smiled and said, \"This is a rabbit. I have a big surprise for you!\".\nThe lion then looked at them. He was holding a big smile. \"I have a surprise, rabbit! I can show you a rabbit.\" the lion said.\nThe rabbit watched the rabbit"

#### rounding: generation 2, seed 2016

Prompt: "Once there was a pink goose. She was very pretty. The goose liked to eat grass, corn and bugs. One"

Stop: {"reason": "eos"}

    "day, they saw a big cake in the garden.\nThe goose and corn went over the cake. It was very colorful and had a big sun with a flag.\nThey found a pretty paper with many leaves and colors. They wanted to eat it, so they started to eat.\nBut then, a big wave came and blew. The goose's little sister, the pig, took the paper to the cake. The goose was very happy. It took the cake from the cake. Everyone was happy. The goose's sister was not happy anymore."

#### rounding: generation 3, seed 2017

Prompt: "One day, a boy named Tim wanted to ride his bike. He loved to ride up the big hill near his house"

Stop: {"reason": "maximum_new_tokens"}

    ". Tim was very excited to show his bike.\nTim rode the bike very fast. The big hill goes fast and loud. Tim was very fast. The big hill hit the bike and it started to go on the trip. Tim's mom said, \"Tim, you're very careful.\"\n Tim did not like the bike. He rode ride, but then he saw that there was a big surprise. He found out that the bike was in his room. Tim was very happy. He showed Tim his bike and he went to the park.\nAt the park, Tim saw a new toy. He felt happy. The next day"

#### rounding: generation 4, seed 2018

Prompt: "One day, there was a small and modest train. Inside the train were two friends - Lily and Dave. They"

Stop: {"reason": "eos"}

    "wanted to make a cake for their friend, Sam, and their home. \nWhen Sam got the new train, they found something shiny in the ground. It was a small, old cake. Sam said, \"This is my cake! I want to keep it for the party, Sam.\"\nSo, Sam and Dave got home and sat down in the ground. They made a delicious cake for Sam's birthday. Sam gave a big smile, and Sam was very happy. They both had a lot of fun and had lots of fun."
