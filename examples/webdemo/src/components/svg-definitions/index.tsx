import { ReactNode } from "react";

function SvgDefinitions({ children }: { children: ReactNode }) {
    return <svg style={{ display: "none" }}>
        <defs>
            {children}
        </defs>
    </svg>;
}

export default SvgDefinitions;