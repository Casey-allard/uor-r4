// This function is serialized into the actual page by Playwright. Keep it closed.
// Finished filling effects are not movement. Only the stylesheet's 0.01 ms CSS
// effects may settle; a long, delayed or repeating effect must never wait to pass.
export async function observeReducedMotion() {
  if (!matchMedia('(prefers-reduced-motion: reduce)').matches) throw Error('Reduced motion: preference is not active');
  const root = document.querySelector('dialog section');
  if (!root) throw Error('Reduced motion: application section is missing');
  return new Promise((resolve, reject) => {
    let frame, samples = 0, quiet = 0, maximumSettlingEffects = 0, last = [];
    const describe = () => JSON.stringify(last, (_, value) => typeof value === 'number' && !Number.isFinite(value) ? String(value) : value);
    const finish = (error) => {
      clearTimeout(deadline);
      if (frame !== undefined) cancelAnimationFrame(frame);
      if (error) reject(error);
      else resolve({samples, maximumSettlingEffects, finishedEffects: last.filter(effect => effect.playState === 'finished' && !effect.pending).length});
    };
    // A wall-clock deadline also fails if the browser stops delivering frames.
    const deadline = setTimeout(() => finish(Error('Reduced motion: settlement timeout ' + describe())), 1000);
    const sample = () => {
      try {
        if (!root.isConnected) throw Error('Reduced motion: application section was detached');
        last = root.getAnimations({subtree: true}).map(animation => {
          const timing = animation.effect?.getComputedTiming?.();
          return {
            kind: animation.constructor.name, name: animation.animationName, property: animation.transitionProperty,
            target: animation.effect?.target?.tagName, pseudoElement: animation.effect?.pseudoElement,
            playState: animation.playState, pending: animation.pending, currentTime: animation.currentTime,
            playbackRate: animation.playbackRate, duration: timing?.duration, delay: timing?.delay,
            endDelay: timing?.endDelay, iterations: timing?.iterations, endTime: timing?.endTime, fill: timing?.fill,
          };
        });
        samples++;
        const active = last.filter(effect => effect.playState !== 'finished' || effect.pending);
        for (const effect of active) {
          if (!['CSSAnimation', 'CSSTransition'].includes(effect.kind) || effect.playState !== 'running' || effect.playbackRate !== 1 ||
              !Number.isFinite(effect.duration) || effect.duration < 0 || effect.duration > 0.01 ||
              effect.delay !== 0 || effect.endDelay !== 0 || effect.iterations !== 1 ||
              !Number.isFinite(effect.endTime) || effect.endTime < 0 || effect.endTime > 0.01) {
            throw Error('Reduced motion: active effect ' + describe());
          }
        }
        maximumSettlingEffects = Math.max(maximumSettlingEffects, active.length);
        quiet = active.length ? 0 : quiet + 1;
        // The initial style-flushing sample plus two quiet painted samples.
        if (quiet >= 3) finish();
        else frame = requestAnimationFrame(sample);
      } catch (error) { finish(error); }
    };
    sample();
  });
}
