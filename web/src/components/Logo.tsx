import logoUrl from "../assets/logo.svg";

/** The KallBET logo, a static asset (currently a placeholder wordmark). */
export function Logo({ className = "h-8 w-auto" }: { className?: string }) {
  return (
    <img
      src={logoUrl}
      alt="KallBET"
      width={332}
      height={80}
      draggable={false}
      className={`select-none ${className}`}
    />
  );
}
