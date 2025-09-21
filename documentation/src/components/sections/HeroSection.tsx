import styles from "../../styles/hero_bg.module.css";
import ChronoGrapherAsset from "../ChronographerAsset.tsx";

export default function HeroSection() {
    return (
        <div class={"bg-gradient-to-b from-base-50 to-transparent h-screen w-screen relative"}>
            <div class={styles["hero-bg"]}/>
            <div class={"absolute size-32 top-0 left-0 bg-primary blur-[120px] rounded-full m-16"}/>
            <div class={"absolute size-36 bottom-0 right-0 bg-secondary blur-[140px] rounded-full mb-36 ml-12"}/>
            <div class={"w-full h-full flex justify-center items-center flex-col relative"}>
                <div class={"size-24 rounded-xl bg-gradient-to-br from-accent to-primary/50 p-[1px] z-10 backdrop-blur-2xl"}>
                    <div class={
                        "rounded-xl bg-accent-content/90 flex justify-center items-center p-2 " +
                        "text-transparent stroke-[0.04rem] stroke-accent/80 *:drop-shadow-md *:drop-shadow-accent/50"
                    }>
                        <ChronoGrapherAsset width={"100%"} height={"100%"} />
                    </div>
                </div>
                <h1 class={
                    "text-[5.3rem] leading-[7rem] tracking-wider font-[Sairia_Stencil_One] " +
                    "z-10 bg-text-clip text-transparent inline-block bg-gradient-to-r from-primary " +
                    "to-secondary drop-shadow-[0_0px_35px_var(--color-primary)] -mt-4 select-none"
                } style={{"-webkit-background-clip": "text"}}>ChronoGrapher</h1>
                <div class={"bg-gradient-to-r from-primary to-secondary opacity-30 w-xl h-1 mb-2"}/>
                <div class={"text-4xl text-secondary/40 font-[Rubik] z-10"}>Enterprise Scheduling Done Right</div>
                <div class={"flex justify-center gap-6 mt-4"}>
                    <button class={"btn btn-primary btn-outline bg-primary/10 hover:bg-primary backdrop-blur-md z-10 group"}>
                        <div class={"text-2xl group-hover:scale-110 transition duration-300"}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24">
                                <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2">
                                    <path d="M11 21.73a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73zm1 .27V12"/>
                                    <path d="M3.29 7L12 12l8.71-5M7.5 4.27l9 5.15"/>
                                </g>
                            </svg>
                        </div>
                        <div class={"font-[Rubik] font-bold"}>Getting Started</div>
                    </button>
                    <button class={"btn btn-warning btn-outline bg-warning/10 hover:bg-warning backdrop-blur-md z-10 group"}>
                        <div class={"text-2xl group-hover:scale-110 transition duration-300"}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24">
                                <path fill="none" stroke="currentColor" stroke-width="1.5" d="M9.153 5.408C10.42 3.136 11.053 2 12 2s1.58 1.136 2.847 3.408l.328.588c.36.646.54.969.82 1.182s.63.292 1.33.45l.636.144c2.46.557 3.689.835 3.982 1.776c.292.94-.546 1.921-2.223 3.882l-.434.507c-.476.557-.715.836-.822 1.18c-.107.345-.071.717.001 1.46l.066.677c.253 2.617.38 3.925-.386 4.506s-1.918.051-4.22-1.009l-.597-.274c-.654-.302-.981-.452-1.328-.452s-.674.15-1.328.452l-.596.274c-2.303 1.06-3.455 1.59-4.22 1.01c-.767-.582-.64-1.89-.387-4.507l.066-.676c.072-.744.108-1.116 0-1.46c-.106-.345-.345-.624-.821-1.18l-.434-.508c-1.677-1.96-2.515-2.941-2.223-3.882S3.58 8.328 6.04 7.772l.636-.144c.699-.158 1.048-.237 1.329-.45s.46-.536.82-1.182z"/>
                            </svg>
                        </div>
                        <div class={"font-[Rubik] font-bolder"}>Support / Star</div>
                    </button>
                </div>
                <div class={"min-w-96 min-h-96 size-55/100 max-w-[100rem] absolute opacity-30 blur-sm text-transparent stroke-primary/60 stroke-[0.3px]"}>
                    <ChronoGrapherAsset width={"100%"} height={"100%"} />
                </div>
            </div>
        </div>
    );
}