import { source } from '@/lib/source';
import { DocsLayout } from 'fumadocs-ui/layouts/docs';
import { baseOptions } from '@/lib/layout.shared';
import ProgrammingLangToggle from "@/components/ui/toggle-language";

export default function Layout({ children }: LayoutProps<'/docs'>) {
  return (
    <DocsLayout tree={source.getPageTree()} sidebar={{
        banner: (
            <ProgrammingLangToggle />
        ),
    }} {...baseOptions()}>
      {children}
    </DocsLayout>
  );
}
