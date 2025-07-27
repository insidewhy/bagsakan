export interface User {
  id: number;
  name: string;
  email: string;
  isActive: boolean;
  tags?: string[];
}

export interface Product {
  id: number;
  name: string;
  price: number;
  description?: string;
  categories: string[];
  inStock: boolean;
}

export interface Order {
  id: number;
  userId: number;
  products: Product[];
  total: number;
  status: 'pending' | 'processing' | 'completed' | 'cancelled';
  createdAt: string;
}
