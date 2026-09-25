/// <reference types="vinxi/types/client" />

interface ImportMetaEnv {
	readonly VITE_SOLID_DEVTOOLS?: string;
}

interface ImportMeta {
	readonly env: ImportMetaEnv;
}
