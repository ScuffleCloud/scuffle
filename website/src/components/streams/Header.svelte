<script lang="ts">
    function createNewStream() {
        console.log('Create new stream');
    }
</script>

<div class="header">
    <div class="title-section">
        <h1>Tab Headline</h1>
        <p>
            We're on a mission to revolutionize video streaming solutions with cutting-edge tools
            and libraries.
        </p>
    </div>

    <a href="/video/streams/new" class="create-button-link">
        <button class="create-button">
            Create New
            <svg
                xmlns="http://www.w3.org/2000/svg"
                width="20"
                height="20"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                <circle cx="12" cy="12" r="10"></circle>
                <path d="M12 8v8"></path>
                <path d="M8 12h8"></path>
            </svg>
        </button>
    </a>
</div>

<style>
    .header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        margin-bottom: 2rem;

        .title-section {
            h1 {
                font-size: 2.5rem;
                font-weight: 700;
                color: #1a1a1a;
                margin: 0 0 0.5rem 0;
            }

            p {
                font-size: 1rem;
                color: #555;
                max-width: 600px;
                margin: 0;
                line-height: 1.5;
            }
        }

        .create-button-link {
            text-decoration: none;

            .create-button {
                display: flex;
                align-items: center;
                gap: 0.5rem;
                background-color: #ffd280;
                color: #000;
                border: none;
                border-radius: 2rem;
                padding: 0.75rem 1.25rem;
                font-weight: 600;
                cursor: pointer;
                transition: background-color 0.2s;

                &:hover {
                    background-color: #ffc966;
                }

                svg {
                    stroke-width: 2.5;
                }
            }
        }
    }
</style>
