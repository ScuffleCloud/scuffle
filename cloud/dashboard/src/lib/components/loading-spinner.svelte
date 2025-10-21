<div class="loading-spinner">
    <div class="spinner"></div>
</div>

<style>
    .loading-spinner {
      display: flex;
      justify-content: center;
      align-items: center;
    }

    .spinner {
      width: 1rem;
      height: 1rem;
      border: 2px solid rgba(255, 255, 255, 0.3);
      border-top: 2px solid black;
      border-radius: 50%;
      animation: spin 1s linear infinite;
    }

    @keyframes spin {
      to {
        transform: rotate(360deg);
      }
    }
</style>
