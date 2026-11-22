// One approval at a time. Forge asks a person before a preview may reach the network, before a request goes
// to an AI provider, and before app source goes to the hosted LexLean service. Each asks in a modal dialog of
// its own, and a modal opened later lies over one opened earlier: the earlier one's buttons are then covered,
// its question goes unanswered, and it expires. Here a question waits until the one before it is answered.
// Its own time limit starts when it is shown, not while it waits.
//
// `open` shows the dialog and returns the promise it settles. It is called at once when nothing is being
// asked, so a caller that asks alone sees no change in timing.
let asking = false;
const waiting = [];

export function oneAtATime(open) {
  return new Promise((resolve, reject) => {
    const start = () => {
      asking = true;
      let answer;
      try { answer = Promise.resolve(open()); } catch (error) { answer = Promise.reject(error); }
      const next = () => { asking = false; const following = waiting.shift(); if (following) following(); };
      answer.then((value) => { next(); resolve(value); }, (error) => { next(); reject(error); });
    };
    if (asking) waiting.push(start); else start();
  });
}

/** How many questions are waiting behind the one being asked. For tests. */
export const waitingCount = () => waiting.length;
