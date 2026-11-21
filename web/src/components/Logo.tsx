import logoUrl from "../assets/logo.svg";

/** The KallBET logo, a static asset (currently a placeholder wordmark). */
export function Logo({ className }: { className?: string }) {
  return <img src={logoUrl} alt="KallBET" className={className} draggable={false} />;
}
