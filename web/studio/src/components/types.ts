export type StudioSection = 'agent' | 'workspace' | 'changes' | 'pr' | 'artifacts' | 'terminal' | 'models' | 'guide' | 'browser';

export interface FileNode {
    path: string;
    name: string;
    kind: 'file' | 'directory';
    sizeBytes?: number;
}

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
