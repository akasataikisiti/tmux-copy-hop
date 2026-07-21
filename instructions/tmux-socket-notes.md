# tmux のソケットと `-S` オプションのメモ

## 対象コマンド

```sh
tmux -S /tmp/tmux-copy-hop-tab.sock copy-mode -t tchtab:0.0
```

このコマンドは、`/tmp/tmux-copy-hop-tab.sock` という専用ソケットで動いている tmux サーバーに接続し、`tchtab` セッションの `0` 番ウィンドウの `0` 番ペインを copy-mode に入れる。

## tmux はなぜサーバーなのか

tmux は単に端末の中で動く表示アプリではなく、クライアント・サーバー方式で動く。

```text
tmux コマンド
  |
  | Unix domain socket
  v
tmux サーバー
  |
  v
セッション / ウィンドウ / ペイン / シェル / スクロールバック
```

役割は大きく分けて次のようになる。

- tmux サーバー
  - セッション、ウィンドウ、ペイン、スクロールバック、実行中のシェルを保持する本体。
  - detach しても裏で動き続ける。
- tmux クライアント
  - `tmux attach`、`tmux copy-mode`、`tmux send-keys` など、ユーザーが実行するコマンド側。
  - サーバーに「このセッションへ接続して」「このペインを copy-mode にして」と依頼する。
- ソケット
  - tmux クライアントが tmux サーバーへ命令を送るための接続口。

tmux で detach しても作業が残るのは、画面を表示していた端末ではなく、裏の tmux サーバーが状態を持ち続けているから。

## Unix domain socket とは

Unix domain socket は、同じマシン上のプロセス同士が通信するための仕組み。

ネットワークの TCP ポートに似ているが、外部ネットワークへ公開するものではなく、ファイルパスのような名前で表される。

例:

```text
/tmp/tmux-copy-hop-tab.sock
```

これは通常のテキストファイルではなく、プロセス間通信の入口。`ls -l` で見ると socket を意味する `s` が表示されることがある。

```sh
ls -l /tmp/tmux-copy-hop-tab.sock
```

Unix domain socket が使われる理由:

- 同じマシン内の通信に向いている。
- ネットワークに公開しなくてよい。
- ファイル権限でアクセス制御しやすい。
- パスで接続先を指定できる。
- 自動テストやスクリプトから扱いやすい。

## `tmux -S` の意味

`-S` は、tmux が使うソケットのパスを明示するオプション。

```sh
tmux -S /tmp/tmux-copy-hop-tab.sock ...
```

これは「通常の tmux サーバーではなく、このソケットで待ち受けている tmux サーバーを操作する」という意味。

普段の tmux は、ユーザーごとのデフォルトソケットを使う。

```sh
tmux
tmux ls
tmux attach
```

一方で `-S` を指定すると、デフォルトとは別の tmux サーバーを作ったり操作したりできる。

```sh
tmux -S /tmp/tmux-copy-hop-tab.sock ls
tmux -S /tmp/tmux-copy-hop-tab.sock attach
tmux -S /tmp/tmux-copy-hop-tab.sock copy-mode -t tchtab:0.0
```

## `/tmp/tmux-copy-hop-tab.sock` はどう作られるか

ソケットは、指定したパスで tmux サーバーを起動したときに作られる。

例:

```sh
tmux -S /tmp/tmux-copy-hop-tab.sock new-session -d -s tchtab
```

このコマンドで起きること:

- `/tmp/tmux-copy-hop-tab.sock` という Unix domain socket が作られる。
- そのソケットを使う tmux サーバーが起動する。
- `tchtab` というセッションが作られる。
- `-d` により、画面には attach せず裏で起動する。

確認:

```sh
tmux -S /tmp/tmux-copy-hop-tab.sock ls
```

終了:

```sh
tmux -S /tmp/tmux-copy-hop-tab.sock kill-server
```

終了すると、その tmux サーバー内のセッション、ウィンドウ、ペイン、実行中プロセスは終了する。

## なぜ検証で専用ソケットを使うのか

`tmux-copy-hop` のような tmux プラグインを検証するとき、普段使っている tmux セッションを直接操作すると危険がある。

たとえば:

- 普段の pane が copy-mode に入る。
- 普段のセッションにキー入力が送られる。
- copy-mode のカーソルや選択状態が変わる。
- テスト用のセッションやペインが混ざる。

専用ソケットを使うと、検証用の tmux サーバーを普段の tmux と分離できる。

```text
普段の tmux
  -> デフォルトソケット

検証用 tmux
  -> /tmp/tmux-copy-hop-tab.sock
```

そのため、Codex やテストスクリプトが次のような流れで検証することがある。

```sh
tmux -S /tmp/tmux-copy-hop-tab.sock new-session -d -s tchtab
tmux -S /tmp/tmux-copy-hop-tab.sock send-keys -t tchtab:0.0 'printf "hello world\n"' Enter
tmux -S /tmp/tmux-copy-hop-tab.sock copy-mode -t tchtab:0.0
tmux -S /tmp/tmux-copy-hop-tab.sock kill-server
```

これは普段の tmux 環境を壊さず、プラグインの挙動だけを独立して確かめるための使い方。

## `-t tchtab:0.0` の意味

`-t` は tmux の対象を指定するオプション。

```sh
-t tchtab:0.0
```

これは次のように分解できる。

```text
tchtab : セッション名
0      : ウィンドウ番号
0      : ペイン番号
```

つまり `tchtab` セッションの `0` 番ウィンドウの `0` 番ペインを対象にしている。

## Linux/Unix では一般的な作りか

このようなクライアント・サーバー方式や Unix domain socket は、Linux/Unix 系のアプリでは珍しくない。

例:

```text
tmux         : tmux CLI -> tmux server
ssh-agent    : ssh / ssh-add -> ssh-agent
gpg-agent    : gpg -> gpg-agent
Docker       : docker CLI -> dockerd
PostgreSQL   : psql -> postgres server
Redis        : redis-cli -> redis-server
Emacs daemon : emacsclient -> emacs --daemon
```

共通点は、状態を持つ本体が裏で動き、CLI はその本体へ命令を送るという構造。

ただし通信方式はアプリによって異なる。Unix domain socket を使うものもあれば、TCP ポート、名前付きパイプ、環境変数で指定された接続先などを使うものもある。

## まとめ

`/tmp/tmux-copy-hop-tab.sock` は、tmux サーバーへ接続するための Unix domain socket。

`tmux -S /tmp/tmux-copy-hop-tab.sock ...` は、普段の tmux ではなく、そのソケットに対応する別の tmux サーバーを操作する。

検証時に専用ソケットを使う主な理由は、普段の tmux 環境とテスト用 tmux 環境を分離するため。

`tmux-copy-hop` のように copy-mode やペイン状態を操作するプラグインでは、この分離が特に重要になる。
