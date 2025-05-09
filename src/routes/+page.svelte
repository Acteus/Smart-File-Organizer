<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";
  import { dndzone } from "svelte-dnd-action";
  import { FolderPlus, FolderSearch, Tags, Upload, Settings, List, Grid, Search, X, Eye, Clock, Cloud, Files } from "lucide-svelte";

  // State management
  let monitoredFolder = $state("");
  let isWatching = $state(false);
  let searchQuery = $state("");
  let selectedTags = $state<number[]>([]);
  let allTags = $state([]);
  let filesList = $state([]);
  let isLoading = $state(false);
  let viewMode = $state("grid"); // grid or list
  let unsubscribe: () => void;

  // Lifecycle
  onMount(async () => {
    // Load tags
    await loadTags();

    // Listen for file events from the backend
    unsubscribe = await listen("file_event", (event) => {
      console.log("File event:", event);
      // Refresh files list when we get a file event
      searchFiles();
    });
  });

  onDestroy(() => {
    // Clean up event listeners
    if (unsubscribe) unsubscribe();
  });

  // Functions
  async function selectFolder() {
    try {
      monitoredFolder = await invoke("select_folder");
    } catch (error) {
      console.error("Error selecting folder:", error);
    }
  }

  async function toggleWatching() {
    if (!monitoredFolder) {
      return;
    }

    isLoading = true;
    try {
      if (!isWatching) {
        await invoke("start_watching_folder", { path: monitoredFolder });
        isWatching = true;
      } else {
        await invoke("stop_watching_folder");
        isWatching = false;
      }
    } catch (error) {
      console.error("Error toggling watch:", error);
    } finally {
      isLoading = false;
    }
  }

  async function handleFileDrop(e: CustomEvent) {
    const items = e.detail.items;
    isLoading = true;
    
    try {
      for (const item of items) {
        await invoke("organize_file", { 
          file_path: item.path,
          destination_folder: null
        });
      }
      // Refresh file list
      await searchFiles();
    } catch (error) {
      console.error("Error processing dropped files:", error);
    } finally {
      isLoading = false;
    }
  }

  async function loadTags() {
    try {
      allTags = await invoke("get_tags");
    } catch (error) {
      console.error("Error loading tags:", error);
    }
  }

  async function searchFiles() {
    isLoading = true;
    try {
      filesList = await invoke("search_files", {
        query: searchQuery || null,
        tagIds: selectedTags.length > 0 ? selectedTags : null,
        extension: null
      });
    } catch (error) {
      console.error("Error searching files:", error);
    } finally {
      isLoading = false;
    }
  }

  function handleTagClick(tagId: number) {
    if (selectedTags.includes(tagId)) {
      selectedTags = selectedTags.filter(id => id !== tagId);
    } else {
      selectedTags = [...selectedTags, tagId];
    }
    searchFiles();
  }

  function clearSearch() {
    searchQuery = "";
    selectedTags = [];
    searchFiles();
  }

  // Format file size helper
  function formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  // File type icons 
  function getFileIcon(extension: string): string {
    const iconMap = {
      pdf: "📄",
      doc: "📄", docx: "📄", txt: "📄", rtf: "📄", odt: "📄",
      jpg: "🖼️", jpeg: "🖼️", png: "🖼️", gif: "🖼️", webp: "🖼️", svg: "🖼️",
      mp4: "🎬", avi: "🎬", mov: "🎬", wmv: "🎬", mkv: "🎬",
      mp3: "🎵", wav: "🎵", flac: "🎵", ogg: "🎵", aac: "🎵",
      zip: "📦", rar: "📦", "7z": "📦", tar: "📦", gz: "📦"
    };
    
    return iconMap[extension.toLowerCase()] || "📁";
  }
</script>

<main class="container mx-auto px-4 py-6 max-w-7xl">
  <header class="mb-8">
    <h1 class="text-3xl font-bold mb-2">Smart File Organizer</h1>
    <p class="text-gray-600 dark:text-gray-400">
      Automatically organize your files with powerful monitoring and tagging
    </p>
  </header>

  <div class="grid grid-cols-1 md:grid-cols-4 gap-6">
    <!-- Sidebar -->
    <aside class="md:col-span-1">
      <div class="card p-4 mb-4">
        <h2 class="text-lg font-semibold mb-4">Folder Monitoring</h2>
        
        <div class="mb-4">
          <button 
            class="btn btn-primary py-2 px-4 w-full mb-2 rounded-md flex items-center justify-center gap-2"
            on:click={selectFolder}
          >
            <FolderPlus size={18} />
            Select Folder
          </button>
          
          {#if monitoredFolder}
            <div class="text-sm text-gray-600 dark:text-gray-400 mb-2 truncate">
              📁 {monitoredFolder}
            </div>
            
            <button 
              class="btn {isWatching ? 'btn-secondary' : 'btn-primary'} py-2 px-4 w-full rounded-md"
              on:click={toggleWatching}
              disabled={isLoading}
            >
              {isWatching ? 'Stop Monitoring' : 'Start Monitoring'}
            </button>
          {/if}
        </div>

        <hr class="my-4 border-gray-200 dark:border-gray-700" />
        
        <h2 class="text-lg font-semibold mb-4">Tags</h2>
        <div class="flex flex-wrap gap-2 mb-4">
          {#each allTags as tag}
            <button 
              class="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium"
              style="background-color: {tag.color}20; color: {tag.color}; border: 1px solid {tag.color}"
              class:ring-2={selectedTags.includes(tag.id)}
              class:ring-offset-2={selectedTags.includes(tag.id)}
              on:click={() => handleTagClick(tag.id)}
            >
              {tag.name}
            </button>
          {/each}
        </div>
        
        <hr class="my-4 border-gray-200 dark:border-gray-700" />
        
        <h2 class="text-lg font-semibold mb-4">Actions</h2>
        <div class="space-y-2">
          <button class="btn btn-outline py-2 px-4 w-full rounded-md flex items-center gap-2">
            <Tags size={16} />
            Manage Tags
          </button>
          <button class="btn btn-outline py-2 px-4 w-full rounded-md flex items-center gap-2">
            <FolderSearch size={16} />
            Organize Rules
          </button>
          <button class="btn btn-outline py-2 px-4 w-full rounded-md flex items-center gap-2">
            <Cloud size={16} />
            Cloud Backup
          </button>
          <button class="btn btn-outline py-2 px-4 w-full rounded-md flex items-center gap-2">
            <Settings size={16} />
            Settings
          </button>
        </div>
      </div>
    </aside>

    <!-- Main Content -->
    <div class="md:col-span-3">
      <!-- Search Bar -->
      <div class="card p-4 mb-6">
        <div class="flex items-center gap-4">
          <div class="relative flex-1">
            <Search size={18} class="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400" />
            <input 
              type="text" 
              class="input pl-10 w-full" 
              placeholder="Search files..." 
              bind:value={searchQuery}
              on:keyup={(e) => e.key === 'Enter' && searchFiles()}
            />
            {#if searchQuery}
              <button 
                class="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400"
                on:click={clearSearch}
              >
                <X size={16} />
              </button>
            {/if}
          </div>
          <button 
            class="btn btn-primary py-2 px-4 rounded-md"
            on:click={searchFiles}
          >
            Search
          </button>
          
          <div class="flex border rounded-md overflow-hidden">
            <button 
              class="p-2 {viewMode === 'grid' ? 'bg-primary-100 text-primary-800' : 'bg-white dark:bg-gray-800'}"
              on:click={() => viewMode = 'grid'}
            >
              <Grid size={18} />
            </button>
            <button 
              class="p-2 {viewMode === 'list' ? 'bg-primary-100 text-primary-800' : 'bg-white dark:bg-gray-800'}"
              on:click={() => viewMode = 'list'}
            >
              <List size={18} />
            </button>
          </div>
        </div>
      </div>

      <!-- Drop Zone -->
      <div 
        class="card p-8 mb-6 border-2 border-dashed border-gray-300 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 flex flex-col items-center justify-center"
        use:dndzone={{items: []}}
        on:consider={handleFileDrop}
        on:finalize={handleFileDrop}
      >
        <Upload size={36} class="text-gray-400 mb-4" />
        <p class="text-center text-gray-600 dark:text-gray-400">
          Drag and drop files here to organize them automatically
        </p>
        <button class="btn btn-primary mt-4 py-2 px-4 rounded-md">
          Browse Files
        </button>
      </div>

      <!-- Files Grid/List -->
      <div class="card p-4">
        <h2 class="text-xl font-semibold mb-4 flex items-center">
          <Files size={20} class="mr-2" />
          Your Files
        </h2>

        {#if isLoading}
          <div class="flex justify-center py-8">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
          </div>
        {:else if filesList.length === 0}
          <div class="py-16 text-center text-gray-500 dark:text-gray-400">
            <p>No files found. Start monitoring a folder or drop files to begin.</p>
          </div>
        {:else if viewMode === 'grid'}
          <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {#each filesList as file}
              <div class="card p-4 hover:shadow-md transition-shadow cursor-pointer">
                <div class="flex items-start mb-2">
                  <div class="text-3xl mr-3">{getFileIcon(file.extension)}</div>
                  <div class="flex-1 min-w-0">
                    <h3 class="font-medium truncate">{file.name}</h3>
                    <p class="text-xs text-gray-500 dark:text-gray-400">{formatFileSize(file.size)}</p>
                  </div>
                </div>
                
                <div class="flex flex-wrap gap-1 mt-2">
                  {#each file.tags as tag}
                    <span 
                      class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium"
                      style="background-color: {tag.color}20; color: {tag.color}; border: 1px solid {tag.color}"
                    >
                      {tag.name}
                    </span>
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <div class="divide-y divide-gray-200 dark:divide-gray-700">
            {#each filesList as file}
              <div class="py-3 flex items-center hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer px-2">
                <div class="text-2xl mr-3">{getFileIcon(file.extension)}</div>
                <div class="flex-1 min-w-0">
                  <h3 class="font-medium truncate">{file.name}</h3>
                  <div class="flex items-center text-xs text-gray-500 dark:text-gray-400">
                    <span class="flex items-center mr-3">
                      <Clock size={12} class="mr-1" />
                      {new Date(file.modified_at).toLocaleDateString()}
                    </span>
                    <span>{formatFileSize(file.size)}</span>
                  </div>
                </div>
                <div class="flex items-center gap-2 ml-4">
                  {#each file.tags as tag}
                    <span 
                      class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium"
                      style="background-color: {tag.color}20; color: {tag.color}; border: 1px solid {tag.color}"
                    >
                      {tag.name}
                    </span>
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
</main>

<style>
.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.svelte-kit:hover {
  filter: drop-shadow(0 0 2em #ff3e00);
}

:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
}

.logo.tauri:hover {
  filter: drop-shadow(0 0 2em #24c8db);
}

.row {
  display: flex;
  justify-content: center;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}
button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

#greet-input {
  margin-right: 5px;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }
}

</style>
