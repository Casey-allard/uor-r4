import { useRef } from 'preact/hooks';

interface SplitterProps { value: number; onChange(value: number): void }

export function Splitter({ value, onChange }: SplitterProps) {
    const drag = useRef<{ startX: number; startValue: number } | null>(null);
    const clamp = (next: number) => Math.max(25, Math.min(75, Math.round(next)));
    return <button type="button" class="pane-splitter" role="separator" aria-label="Resize panes" aria-orientation="vertical" aria-valuemin={25} aria-valuemax={75} aria-valuenow={value}
        onKeyDown={(event) => {
            if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return;
            event.preventDefault();
            if (event.key === 'Home') onChange(25);
            else if (event.key === 'End') onChange(75);
            else onChange(clamp(value + (event.key === 'ArrowRight' ? 5 : -5)));
        }}
        onPointerDown={(event) => { drag.current = { startX: event.clientX, startValue: value }; event.currentTarget.setPointerCapture(event.pointerId); }}
        onPointerMove={(event) => {
            if (!drag.current) return;
            const bounds = event.currentTarget.parentElement?.getBoundingClientRect();
            if (bounds?.width) onChange(clamp(drag.current.startValue + ((event.clientX - drag.current.startX) / bounds.width) * 100));
        }}
        onPointerUp={(event) => { drag.current = null; if (event.currentTarget.hasPointerCapture(event.pointerId)) event.currentTarget.releasePointerCapture(event.pointerId); }}
        onPointerCancel={() => { drag.current = null; }}><span aria-hidden="true" /></button>;
}
