interface EnvironmentPickerProps {
    value: string;
    onChange?(value: string): void;
    disabled?: boolean;
}

export function EnvironmentPicker({ value, onChange, disabled = false }: EnvironmentPickerProps) {
    return <label class="composer-picker">
        <span>Environment</span>
        <select
            aria-label="Composer environment"
            data-composer-control="environment"
            value={value}
            disabled={disabled}
            onChange={(event) => {
                const next = event.currentTarget.value;
                if (next === value) return;
                const option = Array.from(event.currentTarget.options).find((candidate) => candidate.value === next);
                if (!option || option.disabled) {
                    event.currentTarget.value = value;
                    return;
                }
                onChange?.(next);
            }}
        >
            <option value={value}>{value}</option>
            <option value="harness" disabled>Harness service (not connected)</option>
        </select>
    </label>;
}
