import Header from "./components/header";

export default function App() {
  return (
    <div className="relative min-h-screen w-full overflow-x-clip">
      <main className="relative z-10 mx-auto flex flex-col min-h-screen">
        <Header />
      </main>
    </div>
  );
}
