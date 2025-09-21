import PillarCardComponent from "../PillarStrengthCard.tsx";

export default function PillarsSection() {
    return (
        <div class={"flex w-full justify-center flex-col"}>
            <div class={
                "w-full text-center text-[3rem] lg:text-6xl xl:text-7xl leading-none h-32 font-extrabold font-[Rubik] text-transparent " +
                "bg-gradient-to-r from-accent to-secondary bg-text-clip stroke-2 stroke-primary mb-24 select-none"
            } style={{"-webkit-background-clip": "text",}}>
                The ChronoGrapher Advantage
            </div>
            <div class={"flex w-full justify-center items-center"}>
                <div class={"grid grid-cols-2 gap-2 lg:gap-4 xl:gap-x-16 xl:gap-y-4"}>
                    <PillarCardComponent color={"primary"} title={"Designed For Any Scale"} description={
                        "Whenever you are an individual, small team or a global scale company. ChronoGrapher is " +
                        "built for you. Ditching schedulers because of their scalability issues or too much bloat is " +
                        "a thing of the past"
                    } icon={
                        <svg xmlns="http://www.w3.org/2000/svg" width="100%" height="100%" viewBox="0 0 256 256">
                            <path fill="currentColor" d="M200 152a31.84 31.84 0 0 0-19.53 6.68l-23.11-18A31.65 31.65 0 0 0 160 128c0-.74 0-1.48-.08-2.21l13.23-4.41A32 32 0 1 0 168 104c0 .74 0 1.48.08 2.21l-13.23 4.41A32 32 0 0 0 128 96a32.6 32.6 0 0 0-5.27.44L115.89 81A32 32 0 1 0 96 88a32.6 32.6 0 0 0 5.27-.44l6.84 15.4a31.92 31.92 0 0 0-8.57 39.64l-25.71 22.84a32.06 32.06 0 1 0 10.63 12l25.71-22.84a31.91 31.91 0 0 0 37.36-1.24l23.11 18A31.65 31.65 0 0 0 168 184a32 32 0 1 0 32-32m0-64a16 16 0 1 1-16 16a16 16 0 0 1 16-16M80 56a16 16 0 1 1 16 16a16 16 0 0 1-16-16M56 208a16 16 0 1 1 16-16a16 16 0 0 1-16 16m56-80a16 16 0 1 1 16 16a16 16 0 0 1-16-16m88 72a16 16 0 1 1 16-16a16 16 0 0 1-16 16"/>
                        </svg>
                    } />
                    <PillarCardComponent color={"secondary"} title={"Supreme Extensibility"} description={
                        "Tired of working around the limitations of scheduling engines because " +
                        "they lean into their own way of doing things? We too, as such ChronoGrapher is designed to be " +
                        "flexible and extensible "
                    } icon={
                        <svg xmlns="http://www.w3.org/2000/svg" width="100%" height="100%" viewBox="0 0 20 20">
                            <path fill="currentColor" d="M11 3c-.69 0-1.25.56-1.25 1.25V5H6.5a.5.5 0 0 0-.5.5v3.25h-.75a1.25 1.25 0 1 0 0 2.5H6v3.25a.5.5 0 0 0 .5.5h3.25v.75a1.25 1.25 0 1 0 2.5 0V15h3.25a.5.5 0 0 0 .5-.5v-2.25h-.75a2.25 2.25 0 0 1 0-4.5H16V5.5a.5.5 0 0 0-.5-.5h-3.25v-.75C12.25 3.56 11.69 3 11 3M8.764 4a2.25 2.25 0 0 1 4.472 0H15.5A1.5 1.5 0 0 1 17 5.5v3.25h-1.75a1.25 1.25 0 1 0 0 2.5H17v3.25a1.5 1.5 0 0 1-1.5 1.5h-2.264a2.25 2.25 0 0 1-4.472 0H6.5A1.5 1.5 0 0 1 5 14.5v-2.264a2.25 2.25 0 0 1 0-4.472V5.5A1.5 1.5 0 0 1 6.5 4z"/>
                        </svg>
                    } />
                    <PillarCardComponent color={"warning"} title={"Unmatched Performance"} description={
                        "Save costs from power consumption up to hundreds of thousands. ChronoGrapher is written in Rust to ensure " +
                        "maximum performance at any time, making it responsive even under heavy workload"
                    } icon={
                        <svg xmlns="http://www.w3.org/2000/svg" width="100%" height="100%" viewBox="0 0 24 24">
                            <path fill="none" stroke="currentColor" stroke-linejoin="round" stroke-width="1.5" d="M8.628 12.674H8.17c-1.484 0-2.225 0-2.542-.49c-.316-.489-.015-1.17.588-2.533l1.812-4.098c.548-1.239.822-1.859 1.353-2.206S10.586 3 11.935 3h2.09c1.638 0 2.458 0 2.767.535c.309.536-.098 1.25-.91 2.681l-1.073 1.886c-.404.711-.606 1.066-.603 1.358c.003.378.205.726.53.917c.25.147.657.147 1.471.147c1.03 0 1.545 0 1.813.178c.349.232.531.646.467 1.061c-.049.32-.395.703-1.088 1.469l-5.535 6.12c-1.087 1.203-1.63 1.804-1.996 1.613c-.365-.19-.19-.983.16-2.569l.688-3.106c.267-1.208.4-1.812.08-2.214c-.322-.402-.937-.402-2.168-.402Z"/>
                        </svg>
                    } />
                    <PillarCardComponent color={"accent"} title={"Language Agnostic"} description={
                        "Gone are the days of being forced to use one and only programming language for scheduling " +
                        "logic. ChronoGrapher allows the use of multiple programming language "
                    } icon={
                        <svg xmlns="http://www.w3.org/2000/svg" width="100%" height="100%" viewBox="0 0 24 24">
                            <path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2 5h14M9 2v3m4 0q-2 8-9 11m2-7q2 4 6 6m1 7l5-11l5 11m-1.4-3h-7.2"/>
                        </svg>
                    } />
                    <PillarCardComponent color={"info"} title={"Composable Architecture"} description={
                        "Instead of thinking tasks as execution logic and schedulers as the logic for scheduling. " +
                        "ChronoGrapher breaks it up to multiple components, making it composition-based, flexible and adaptable"
                    } icon={
                        <svg xmlns="http://www.w3.org/2000/svg" width="100%" height="100%" viewBox="0 0 256 256">
                            <path fill="currentColor" d="M218.68 125.46a12 12 0 1 0-21.37-10.92a75.15 75.15 0 0 1-27.66 29.64l-13.5-30.39A44 44 0 0 0 140 37.68V24a12 12 0 0 0-24 0v13.68a44 44 0 0 0-16.15 76.11L53 219.12A12 12 0 0 0 59.13 235a11.9 11.9 0 0 0 4.87 1a12 12 0 0 0 11-7.13l23.67-53.26A99.5 99.5 0 0 0 128 180a102.8 102.8 0 0 0 29.39-4.32L181 228.87a12 12 0 0 0 11 7.13a11.85 11.85 0 0 0 4.86-1a12 12 0 0 0 6.14-15.88l-23.51-52.9a99.4 99.4 0 0 0 39.19-40.76M128 60a20 20 0 1 1-20 20a20 20 0 0 1 20-20m0 96a75.8 75.8 0 0 1-19.52-2.53l13.3-29.92a43.2 43.2 0 0 0 12.44 0l13.33 30A79 79 0 0 1 128 156"/>
                        </svg>
                    } />
                    <PillarCardComponent color={"success"} title={"Free & Open Source"} description={
                        "ChronoGrapher is not only free forever but also open source. It is made and maintained by " +
                        "the community. We believe these kinds of projects should always be free to the outer public"
                    } icon={
                        <svg xmlns="http://www.w3.org/2000/svg" width="100%" height="100%" viewBox="0 0 20 20">
                            <path fill="currentColor" d="M4.5 6.75a2.25 2.25 0 1 1 4.5 0a2.25 2.25 0 0 1-4.5 0M6.75 3.5a3.25 3.25 0 1 0 0 6.5a3.25 3.25 0 0 0 0-6.5m5.687 11.645c.538.22 1.215.355 2.063.355c1.881 0 2.921-.668 3.469-1.434a2.9 2.9 0 0 0 .521-1.36a2 2 0 0 0 .01-.137V12.5A1.5 1.5 0 0 0 17 11h-4.63c.24.29.42.629.525 1H17a.5.5 0 0 1 .5.5v.054l-.005.05a1.9 1.9 0 0 1-.34.88c-.327.459-1.037 1.016-2.655 1.016c-.732 0-1.278-.114-1.687-.281c-.082.28-.201.596-.376.926M1.5 13a2 2 0 0 1 2-2H10a2 2 0 0 1 2 2v.084l-.002.04l-.01.135a3.95 3.95 0 0 1-.67 1.806C10.617 16.08 9.263 17 6.75 17s-3.867-.92-4.568-1.934a3.95 3.95 0 0 1-.67-1.807a3 3 0 0 1-.012-.175zm1 .06v.018l.007.083a2.95 2.95 0 0 0 .498 1.336C3.492 15.201 4.513 16 6.75 16s3.258-.799 3.745-1.503a2.95 2.95 0 0 0 .498-1.336q.006-.057.006-.083l.001-.017V13a1 1 0 0 0-1-1H3.5a1 1 0 0 0-1 1zM13 7.5a1.5 1.5 0 1 1 3 0a1.5 1.5 0 0 1-3 0M14.5 5a2.5 2.5 0 1 0 0 5a2.5 2.5 0 0 0 0-5"/>
                        </svg>
                    } />
                    <PillarCardComponent color={"neutral"} title={"Intuitive API"} description={
                        "No complexity is required to use it (as life intended to be), vast documentation on multiple " +
                        "topics and best of all, ChronoGrapher offers a clear developer API "
                    } icon={
                        <svg xmlns="http://www.w3.org/2000/svg" width="100%" height="100%" viewBox="-1 -2 24 24">
                            <path fill="currentColor" d="M14 5.714a1.474 1.474 0 0 0 0 2.572l-.502 1.684a1.473 1.473 0 0 0-1.553 2.14l-1.443 1.122A1.473 1.473 0 0 0 8.143 14l-2.304-.006a1.473 1.473 0 0 0-2.352-.765l-1.442-1.131A1.473 1.473 0 0 0 .5 9.968L0 8.278a1.474 1.474 0 0 0 0-2.555l.5-1.69a1.473 1.473 0 0 0 1.545-2.13L3.487.77A1.473 1.473 0 0 0 5.84.005L8.143 0a1.473 1.473 0 0 0 2.358.768l1.444 1.122a1.473 1.473 0 0 0 1.553 2.14zM7 10a3 3 0 1 0 0-6a3 3 0 0 0 0 6m7.393.061a8 8 0 0 0 .545-4.058L16.144 6a1.473 1.473 0 0 0 2.358.768l1.444 1.122a1.473 1.473 0 0 0 1.553 2.14L22 11.714a1.474 1.474 0 0 0 0 2.572l-.502 1.684a1.473 1.473 0 0 0-1.553 2.14l-1.443 1.122a1.473 1.473 0 0 0-2.359.768l-2.304-.006a1.473 1.473 0 0 0-2.352-.765l-1.442-1.131a1.473 1.473 0 0 0-1.545-2.13l-.312-1.056a7.96 7.96 0 0 0 3.821-1.674a3 3 0 1 0 2.384-3.177"/>
                        </svg>
                    } />
                    <PillarCardComponent color={"error"} title={"Millisecond Precision"} description={
                        "ChronoGrapher also allows for precise millisecond calculations, useful in cases where " +
                        "tasks may require higher timing resolution than what most schedulers offer"
                    } icon={
                        <svg xmlns="http://www.w3.org/2000/svg" width="100%" height="100%" viewBox="0 0 24 24">
                            <path fill="currentColor" d="M12 20a7 7 0 0 1-7-7a7 7 0 0 1 7-7a7 7 0 0 1 7 7a7 7 0 0 1-7 7m0-16a9 9 0 0 0-9 9a9 9 0 0 0 9 9a9 9 0 0 0 9-9a9 9 0 0 0-9-9m.5 4H11v6l4.75 2.85l.75-1.23l-4-2.37zM7.88 3.39L6.6 1.86L2 5.71l1.29 1.53zM22 5.72l-4.6-3.86l-1.29 1.53l4.6 3.86z"/>
                        </svg>
                    } />
                </div>
            </div>
        </div>
    );
}