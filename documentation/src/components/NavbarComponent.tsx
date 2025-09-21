import ChronoGrapherAsset from "./ChronographerLogo.tsx";
import IconButton from "./IconButton.tsx";

export default function NavbarComponent() {
    return (
        <div class={
            "w-full h-18 bg-base-200/80 backdrop-blur-md fixed flex justify-between " +
            "p-1 border-b border-b-neutral-content/50 px-1 pr-4 md:px-2.5 md:pr-6 items-center " +
            "z-100"
        }>
            <div class={"flex gap-1 md:gap-2 items-center"}>
                <ChronoGrapherAsset />
                <label class={"input input-sm md:input-md !outline-none !flex-row-reverse"}>
                    <input type="search" class={"peer"} placeholder="Search" />
                    <div class={"text-xl opacity-30 peer-focus:opacity-100"}>
                        <svg width={"1em"} height={"1em"} xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                            <g
                                stroke-linejoin="round"
                                stroke-linecap="round"
                                stroke-width="2.5"
                                fill="none"
                                stroke="currentColor"
                            >
                                <circle cx="11" cy="11" r="8"></circle>
                                <path d="m21 21-4.3-4.3"></path>
                            </g>
                        </svg>
                    </div>
                </label>
            </div>
            <div class={"flex gap-2 md:gap-4 items-center flex-row-reverse"}>
                <IconButton color={"neutral"} tooltip={"Reference"}>
                    <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24">
                        <path fill="currentColor" d="M8.25 8A.75.75 0 0 1 9 7.25h7a.75.75 0 0 1 0 1.5H9A.75.75 0 0 1 8.25 8M9 10.25a.75.75 0 0 0 0 1.5h5a.75.75 0 0 0 0-1.5z"/>
                        <path fill="currentColor" fill-rule="evenodd" d="M8.5 3.25A4.75 4.75 0 0 0 3.75 8v10a3.75 3.75 0 0 0 3.75 3.75h11A1.75 1.75 0 0 0 20.25 20V5a1.75 1.75 0 0 0-1.75-1.75zm10.25 11V5a.25.25 0 0 0-.25-.25h-10A3.25 3.25 0 0 0 5.25 8v7a3.73 3.73 0 0 1 2.25-.75zm0 1.5H7.5a2.25 2.25 0 0 0 0 4.5h11a.25.25 0 0 0 .25-.25z" clip-rule="evenodd"/>
                    </svg>
                </IconButton>
                <div class={"tooltip tooltip-bottom"} data-tip={"Localization"}>
                    <button class={
                        `btn btn-square btn-soft btn-secondary border-secondary/50 btn-sm text-xl md:btn-md md:text-[1.65rem]
                         border-1 hover:text-2xl md:hover:text-[1.85rem] duration-300 transition-all`
                    }>
                        <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 15 15"><path fill="currentColor" d="M7.5.9a6.6 6.6 0 1 1-.002 13.201A6.6 6.6 0 0 1 7.5.901m2.714 7.05c-.086 1.884-.718 3.752-1.912 5.192A5.7 5.7 0 0 0 13.18 7.95zm-8.395 0a5.7 5.7 0 0 0 4.554 5.137C5.24 11.654 4.643 9.809 4.56 7.95zm3.643 0c.09 1.84.73 3.621 1.893 4.907C8.559 11.57 9.22 9.79 9.313 7.95zM7.355 2.14C6.192 3.427 5.552 5.208 5.462 7.05h3.851c-.093-1.84-.754-3.621-1.958-4.908m.947-.284c1.194 1.44 1.826 3.309 1.912 5.192h2.966a5.7 5.7 0 0 0-4.878-5.192m-1.928.054A5.7 5.7 0 0 0 1.82 7.049h2.74c.083-1.859.681-3.705 1.814-5.138"/></svg>
                    </button>
                </div>
                <IconButton color={"primary"} tooltip={"Installing"}>
                    <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24">
                        <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2">
                            <path d="M11 21.73a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73zm1 .27V12"/>
                            <path d="M3.29 7L12 12l8.71-5M7.5 4.27l9 5.15"/>
                        </g>
                    </svg>
                </IconButton>
                <IconButton color={"warning"} tooltip={"Star Project"}>
                    <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24">
                        <path fill="none" stroke="currentColor" stroke-width="1.5" d="M9.153 5.408C10.42 3.136 11.053 2 12 2s1.58 1.136 2.847 3.408l.328.588c.36.646.54.969.82 1.182s.63.292 1.33.45l.636.144c2.46.557 3.689.835 3.982 1.776c.292.94-.546 1.921-2.223 3.882l-.434.507c-.476.557-.715.836-.822 1.18c-.107.345-.071.717.001 1.46l.066.677c.253 2.617.38 3.925-.386 4.506s-1.918.051-4.22-1.009l-.597-.274c-.654-.302-.981-.452-1.328-.452s-.674.15-1.328.452l-.596.274c-2.303 1.06-3.455 1.59-4.22 1.01c-.767-.582-.64-1.89-.387-4.507l.066-.676c.072-.744.108-1.116 0-1.46c-.106-.345-.345-.624-.821-1.18l-.434-.508c-1.677-1.96-2.515-2.941-2.223-3.882S3.58 8.328 6.04 7.772l.636-.144c.699-.158 1.048-.237 1.329-.45s.46-.536.82-1.182z"/>
                    </svg>
                </IconButton>
            </div>
        </div>
    );
}