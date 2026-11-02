export type StudioSection = 'agent' | 'workspace' | 'models' | 'guide';

export interface ShellStatus {
    source: string;
    artifact: string;
    privacy: string;
    file: string;
    session: string;
}

export interface PaletteCommand {
    id: string;
    group: string;
    label: string;
    description: string;
    shortcut?: string;
    run(): void;
    disabled?: boolean;
}
