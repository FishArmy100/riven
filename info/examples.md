# Examples
### Creating a summing function
```rs
fn sum(self) -> Int for []Int // for complex types, can have a `for` clause
{
    let r = 0;
    for i in 0..self.len()
    {
        r += self[i];
    }

    return r; 
}

let total = [1, 2, 3, 4].sum();
```

### Lambdas
```rs
struct Board
{
    board: []Int = [0; 9]
}

fn Board.iter(self) -> Fn() -> ?Int
{
    let i = 0;
    return || => {
        if i < self.board.len()
        {
            let v = self.board[i];
            i += 1;
            return v;
        }
        else 
        {
            return null;
        }
    };
}

let board = Board {};
for v in board.iter()
{
    print(v.to_string());
}
```