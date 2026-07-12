import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Card, CardHeader, CardContent, CardActions, CardMedia } from './Card';

describe('Card', () => {
  it('renders children', () => {
    render(<Card>Card content</Card>);
    expect(screen.getByText('Card content')).toBeInTheDocument();
  });

  it('renders as div by default', () => {
    render(<Card data-testid="card">Content</Card>);
    expect(screen.getByTestId('card').tagName).toBe('DIV');
  });

  it('renders as custom component', () => {
    render(<Card component="article" data-testid="card">Article</Card>);
    expect(screen.getByTestId('card').tagName).toBe('ARTICLE');
  });

  it('applies overflow-hidden', () => {
    render(<Card data-testid="card">Content</Card>);
    expect(screen.getByTestId('card')).toHaveClass('overflow-hidden');
  });

  it('applies Paper props (elevation)', () => {
    render(<Card elevation={3} data-testid="card">Content</Card>);
    expect(screen.getByTestId('card')).toHaveClass('shadow-md');
  });

  it('applies Paper props (variant outlined)', () => {
    render(<Card variant="outlined" data-testid="card">Content</Card>);
    expect(screen.getByTestId('card')).toHaveClass('border', 'border-nest-border');
  });

  it('applies custom className', () => {
    render(<Card className="custom-card" data-testid="card">Content</Card>);
    expect(screen.getByTestId('card')).toHaveClass('custom-card');
  });

  it('forwards ref', () => {
    const ref = { current: null as HTMLElement | null };
    render(<Card ref={ref} data-testid="card">Content</Card>);
    expect(ref.current).toBeInTheDocument();
  });
});

describe('CardHeader', () => {
  it('renders as div by default', () => {
    render(<CardHeader title="Title" data-testid="header" />);
    expect(screen.getByTestId('header').tagName).toBe('DIV');
  });

  it('renders as custom component', () => {
    render(<CardHeader component="header" title="Title" data-testid="header" />);
    expect(screen.getByTestId('header').tagName).toBe('HEADER');
  });

  it('renders title', () => {
    render(<CardHeader title="My Title" data-testid="header" />);
    expect(screen.getByText('My Title')).toHaveClass('font-semibold', 'text-nest-foreground');
  });

  it('renders subheader', () => {
    render(<CardHeader title="Title" subheader="My Subheader" data-testid="header" />);
    expect(screen.getByText('My Subheader')).toHaveClass('text-sm', 'text-nest-muted');
  });

  it('renders avatar', () => {
    render(<CardHeader title="Title" avatar={<span data-testid="avatar">👤</span>} data-testid="header" />);
    expect(screen.getByTestId('avatar')).toBeInTheDocument();
  });

  it('renders action', () => {
    render(<CardHeader title="Title" action={<button data-testid="action">Action</button>} data-testid="header" />);
    expect(screen.getByTestId('action')).toBeInTheDocument();
  });

  it('renders all elements together', () => {
    render(
      <CardHeader
        avatar={<span data-testid="avatar">👤</span>}
        title="Title"
        subheader="Subtitle"
        action={<button data-testid="action">✕</button>}
        data-testid="header"
      />
    );
    expect(screen.getByTestId('avatar')).toBeInTheDocument();
    expect(screen.getByText('Title')).toBeInTheDocument();
    expect(screen.getByText('Subtitle')).toBeInTheDocument();
    expect(screen.getByTestId('action')).toBeInTheDocument();
  });

  it('applies flex layout', () => {
    render(<CardHeader title="Title" data-testid="header" />);
    expect(screen.getByTestId('header')).toHaveClass('flex', 'items-start', 'gap-3', 'p-4');
  });

  it('applies custom className', () => {
    render(<CardHeader title="Title" className="custom-header" data-testid="header" />);
    expect(screen.getByTestId('header')).toHaveClass('custom-header');
  });

  it('avatar is shrink-0', () => {
    render(<CardHeader title="Title" avatar={<span data-testid="avatar">A</span>} data-testid="header" />);
    expect(screen.getByTestId('avatar').parentElement).toHaveClass('shrink-0');
  });

  it('action is shrink-0', () => {
    render(<CardHeader title="Title" action={<button data-testid="action">✕</button>} data-testid="header" />);
    expect(screen.getByTestId('action').parentElement).toHaveClass('shrink-0');
  });
});

describe('CardContent', () => {
  it('renders children', () => {
    render(<CardContent>Content</CardContent>);
    expect(screen.getByText('Content')).toBeInTheDocument();
  });

  it('renders as div by default', () => {
    render(<CardContent data-testid="content">Content</CardContent>);
    expect(screen.getByTestId('content').tagName).toBe('DIV');
  });

  it('renders as custom component', () => {
    render(<CardContent component="section" data-testid="content">Section</CardContent>);
    expect(screen.getByTestId('content').tagName).toBe('SECTION');
  });

  it('applies padding', () => {
    render(<CardContent data-testid="content">Content</CardContent>);
    expect(screen.getByTestId('content')).toHaveClass('p-4');
  });

  it('applies custom className', () => {
    render(<CardContent className="custom-content" data-testid="content">Content</CardContent>);
    expect(screen.getByTestId('content')).toHaveClass('custom-content');
  });

  it('forwards ref', () => {
    const ref = { current: null as HTMLElement | null };
    render(<CardContent ref={ref} data-testid="content">Content</CardContent>);
    expect(ref.current).toBeInTheDocument();
  });
});

describe('CardActions', () => {
  it('renders children', () => {
    render(<CardActions><button>Action</button></CardActions>);
    expect(screen.getByText('Action')).toBeInTheDocument();
  });

  it('renders as div by default', () => {
    render(<CardActions data-testid="actions"><button>Action</button></CardActions>);
    expect(screen.getByTestId('actions').tagName).toBe('DIV');
  });

  it('renders as custom component', () => {
    render(<CardActions component="footer" data-testid="actions"><button>Action</button></CardActions>);
    expect(screen.getByTestId('actions').tagName).toBe('FOOTER');
  });

  it('applies default spacing (gap-2)', () => {
    render(<CardActions data-testid="actions"><button>A</button><button>B</button></CardActions>);
    expect(screen.getByTestId('actions')).toHaveClass('gap-2');
  });

  it('removes spacing when disableSpacing=true', () => {
    render(<CardActions disableSpacing data-testid="actions"><button>A</button><button>B</button></CardActions>);
    expect(screen.getByTestId('actions')).not.toHaveClass('gap-2');
  });

  it('applies flex layout', () => {
    render(<CardActions data-testid="actions"><button>Action</button></CardActions>);
    expect(screen.getByTestId('actions')).toHaveClass('flex', 'items-center', 'p-2');
  });

  it('applies custom className', () => {
    render(<CardActions className="custom-actions" data-testid="actions"><button>Action</button></CardActions>);
    expect(screen.getByTestId('actions')).toHaveClass('custom-actions');
  });
});

describe('CardMedia', () => {
  it('renders as div by default', () => {
    render(<CardMedia data-testid="media" />);
    const div = screen.getByTestId('media');
    expect(div.tagName).toBe('DIV');
    expect(div).toHaveClass('bg-nest-muted/20');
  });

  it('renders as img when image prop provided', () => {
    render(<CardMedia image="/test.jpg" alt="Test" data-testid="media" />);
    const img = screen.getByTestId('media');
    expect(img.tagName).toBe('IMG');
    expect(img).toHaveAttribute('src', '/test.jpg');
    expect(img).toHaveAttribute('alt', 'Test');
  });

  it('renders as img when component=img', () => {
    render(<CardMedia component="img" image="/photo.png" alt="Photo" data-testid="media" />);
    const img = screen.getByTestId('media');
    expect(img.tagName).toBe('IMG');
  });

  it('applies default height (140px)', () => {
    render(<CardMedia image="/test.jpg" data-testid="media" />);
    const img = screen.getByTestId('media');
    expect(img).toHaveClass('h-[--card-media-height]');
  });

  it('applies custom height', () => {
    render(<CardMedia image="/test.jpg" height="200px" data-testid="media" />);
    const img = screen.getByTestId('media');
    expect(img).toHaveClass('h-[--card-media-height]');
  });

  it('applies object-cover for img', () => {
    render(<CardMedia image="/test.jpg" data-testid="media" />);
    const img = screen.getByTestId('media');
    expect(img).toHaveClass('object-cover');
  });

  it('applies bg-nest-muted/20 for div', () => {
    render(<CardMedia data-testid="media" />);
    const div = screen.getByTestId('media');
    expect(div).toHaveClass('bg-nest-muted/20');
  });

  it('applies w-full', () => {
    render(<CardMedia image="/test.jpg" data-testid="media" />);
    const img = screen.getByTestId('media');
    expect(img).toHaveClass('w-full');
  });

  it('applies custom className', () => {
    render(<CardMedia image="/test.jpg" className="custom-media" data-testid="media" />);
    const img = screen.getByTestId('media');
    expect(img).toHaveClass('custom-media');
  });

  it('passes through title prop', () => {
    render(<CardMedia image="/test.jpg" title="Test Title" data-testid="media" />);
    const img = screen.getByTestId('media');
    expect(img).toHaveAttribute('title', 'Test Title');
  });
});
