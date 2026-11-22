export type PermissionMode = 'ask' | 'autonomous';

interface PermissionPickerProps {
    mode: PermissionMode;
    onChange?(mode: PermissionMode): void;
    hostAvailable?: boolean;
}

export function PermissionPicker({ mode, onChange, hostAvailable = false }: PermissionPickerProps) {
    return <label class="composer-picker">
        <span>Permission</span>
        <select
            aria-label="Permission mode"
            data-composer-control="permission"
            value={mode}
            onChange={(event) => {
                const next = event.currentTarget.value as PermissionMode;
                if (next === 'autonomous' && !hostAvailable) {
                    event.currentTarget.value = mode;
                    return;
                }
                onChange?.(next);
            }}
        >
            <option value="ask">Ask before write</option>
            <option value="autonomous" disabled={!hostAvailable}>Autonomous (requires host)</option>
        </select>
    </label>;
}
