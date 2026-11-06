
export default function EmptyState({ icon, message, action }: { icon?: string; message: string; action?: React.ReactNode }) {
  return (
    <div className="empty">
      {icon && <div className="icon">{icon}</div>}
      <p>{message}</p>
      {action}
    </div>
  );
}
