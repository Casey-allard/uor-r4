import { useState } from 'preact/hooks';
import { Modal } from './Modal.js';
import type { PaletteCommand } from './types.js';

interface CommandPaletteProps {
    commands: PaletteCommand[];
    onClose(): void;
}

export function CommandPalette({ commands, onClose }: CommandPaletteProps) {
    const [query, setQuery] = useState('');
    const [active, setActive] = useState(0);
    const normalized = query.trim().toLowerCase();
    const filtered = commands.filter((command) => `${command.group} ${command.label} ${command.description}`.toLowerCase().includes(normalized));
    const activeIndex = Math.max(0, Math.min(active, filtered.length - 1));

    function run(command: PaletteCommand | undefined) {
        if (!command || command.disabled) return;
        command.run();
        onClose();
    }

    return (
        <Modal title="Command palette" onClose={onClose}>
            <input
                type="search"
                aria-label="Search commands"
                autofocus
                value={query}
                onInput={(event) => { setQuery(event.currentTarget.value); setActive(0); }}
                onKeyDown={(event) => {
                    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
                        event.preventDefault();
                        if (!filtered.length) return;
                        const delta = event.key === 'ArrowDown' ? 1 : -1;
                        setActive((activeIndex + delta + filtered.length) % filtered.length);
                    } else if (event.key === 'Enter') {
                        event.preventDefault();
                        run(filtered[activeIndex]);
                    }
                }}
            />
            <p>{filtered.length ? `${filtered.length} commands` : 'No matching commands'}</p>
            <ul>
                {filtered.map((command, index) => (
                    <li>
                        <button type="button" data-command-id={command.id} data-active={index === activeIndex ? 'true' : undefined} disabled={command.disabled} onClick={() => run(command)}>
                            <strong>{command.label}</strong>
                            <span>{command.description}</span>
                            <small>{command.group}{command.shortcut ? ` · ${command.shortcut}` : ''}</small>
                        </button>
                    </li>
                ))}
            </ul>
        </Modal>
    );
}
