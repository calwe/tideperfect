import { HashRouter, Routes, Route } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import Home from './home/Home';
import Login from './login/Login';
import Album from './album/Album';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: false,
      refetchOnWindowFocus: false,
    },
  },
});

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <main>
        <HashRouter>
          <Routes>
            <Route path="/" element={<Home />} />
            <Route path="/login" element={<Login />} />
            <Route path="/album/:albumId" element={<Album />} />
          </Routes>
        </HashRouter>
      </main>
    </QueryClientProvider>
  );
}

export default App;
