// Catalog cache + current document (Svelte 5 runes). The Rust core is the
// source of truth; this mirrors it for rendering.
import type { DocMeta } from "../bindings";
import { listDocuments } from "../api";

class DocumentsStore {
  catalog = $state<DocMeta[]>([]);
  /** Id of the doc open in the editor; null = new unsaved doc. */
  openId = $state<string | null>(null);

  async refresh() {
    this.catalog = await listDocuments();
  }
}

export const documents = new DocumentsStore();
