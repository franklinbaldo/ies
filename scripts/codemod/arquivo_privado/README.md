---
type: Guide
title: Codemod do ErroArquivoPrivado
description: Pipeline reproduzível (ast-grep e ajustes listados) que tira os erros de arquivo de FalhaTransporte, apaga os envelopes de uma variante só e entrega à escrita o cookie que a abertura conferiu
---

# Codemod `ErroArquivoPrivado`

A leitura de arquivo privado produzia nove variantes de `FalhaTransporte`
(`ArquivoPrivado*` e `CaminhoDeArquivoNaoEhArquivo`), e quem a usava
embrulhava a falha em enums de uma variante só (`CookieIlegivel`,
`CredenciaisIlegiveis`, `OrigemIlegivel`, `SessaoGravadaInvalida`). Os
comandos ainda reliam o cookie depois da abertura, que já o tinha lido, e
carregavam para isso uma variante que nenhum caminho produzia. O pipeline
refaz a mudança inteira a partir de qualquer HEAD limpo:

```sh
scripts/codemod/arquivo_privado/pipeline.sh                  # tudo
scripts/codemod/arquivo_privado/pipeline.sh --ate-as-regras  # para antes dos ajustes
```

Depois: `cargo clippy`, `cargo test` e o replay, como em qualquer mudança.

## Desenho

- `arquivo_privado::ler(caminho, limite) -> Result<String, ErroArquivoPrivado>`
  (`src/arquivo_privado/leitura.rs`, de `modelos/leitura.rs`). O corpo é
  o da antiga `transporte::arquivo_privado`.
- `ErroArquivoPrivado`: `Ausente` (só `NotFound`),
  `MetadadosIlegiveis(ErrorKind)`, `LinkSimbolico`, `NaoRegular`,
  `PermissoesAbertas` (só Unix), `Vazio`, `AcimaDoLimite`,
  `AberturaRecusada(ErrorKind)`, `ConteudoIlegivel(ErrorKind)` e
  `NaoUtf8(Utf8Error)`.
- O antigo `configuracao::ErroArquivoPrivado`, que só confere metadados
  dos arquivos de acesso, vira `ErroArquivoDeAcesso`.
- `FalhaTransporte` perde as nove variantes e embrulha uma vez, com o nome
  do arquivo que o transporte lê: `UrlDoControleIlegivel`,
  `OrigemIlegivel`, `CookieDaSessaoIlegivel` e `MarcadorDaSessaoIlegivel`.
- `CookieDaSessao` diz o fato: `Lido(CookieLido)`,
  `Ilegivel(ErroArquivoPrivado)`, `ForaDoFormato` ou
  `NaoAberto(FalhaTransporte)` (a origem recusada). `CookieLido` é o
  cookie conferido, que só o retrato cria.
- A abertura (`abrir_para_escrever`, a abertura do documento e a troca de
  unidade) lê o cookie uma vez e o entrega em `AcoesDeEstado`,
  `DocumentoNaArvoreAberto`, `DocumentoObtido` e `Troca`. `ler_cookie`, a
  variante `Cookie` dos 19 enums de comando, `ErroSalvarNoSei::Cookie` e
  `FalhaAplicarTroca` saem.
- `processo criar` lê o cookie primeiro: `ErroCriar` ganha
  `CookieDaSessaoIlegivel(ErroArquivoPrivado)`, `CookieForaDoFormato` e
  `ControleForaDaOrigem(FalhaTransporte)`.
- `ErroLogin` leva o erro do arquivo sem envelope: `ArquivoDeCredenciais`,
  `OrigemIlegivel`, `ArquivoDaSessao` e `OrigemNaoAutorizada` (origem lida
  que não é instalação).

## Passos

| Passo | Peça | O que faz |
| --- | --- | --- |
| 0 | `modelos/funcao-antiga.yml` | confere que a leitura em `transporte.rs` ainda é `modelos/arquivo_privado_antigo.rs.txt`; se mudou, para |
| 1 | `regras/05-erro-de-acesso` | `ErroArquivoPrivado` de `src/configuracao*` vira `ErroArquivoDeAcesso` (também em macro) |
| 2 | `regras/10-cookie-da-abertura` | `ler_cookie(s)?` numa função que chama `abrir_para_escrever` vira `acoes.cookie.clone()`; com `abrir_documento_na_arvore`, `aberto.cookie.clone()` |
| 2 | `regras/11-variante-do-cookie-sai` | apaga `Cookie(CookieIlegivel)`, o comentário dela e o `impl From<CookieIlegivel>` |
| 2 | `regras/12-usos-da-variante-do-cookie` | tira `X::Cookie(_)` do padrão "ou" e `X::Cookie(CookieIlegivel::Arquivo(..))` da lista do teste |
| 2 | `regras/14-import-cookie-ilegivel` | tira `CookieIlegivel` e `ler_cookie` dos `use` |
| 3 | `regras/30-leitura-sai-do-transporte` | apaga `transporte::arquivo_privado`; chamadas (também em macro) viram `crate::arquivo_privado::ler`; sai do `use super::{…}` |
| 3 | `regras/31-variantes-do-arquivo` | `FalhaTransporte::ArquivoPrivado*` → `crate::arquivo_privado::ErroArquivoPrivado::*`; `ArquivoPrivadoAusenteOuInacessivel(_)` → `Ausente` (também em macro) |
| 4 | `regras/40-testes-da-cli` | nos testes da CLI sem sessão: `falha(s, "ArquivoPrivadoAusenteOuInacessivel")` → `falha_por(s, "UrlDoControleIlegivel", "Ausente")`; o prefixo `Error: …` vira o Debug novo |
| 5 | `modelos/leitura.rs` | vira `src/arquivo_privado/leitura.rs` |
| 6 | `ajustes.py` | o que nenhuma regra cobre (abaixo) |
| 7 | `../html5ever/imports_sem_uso.py`, `cargo fmt` | tira os `use` que ficaram sem uso |

As regras que apagam uma linha deixam `/* ies-codemod: remover linha */`, e
o pipeline apaga a linha no fim do passo, como no codemod do html5ever.

## Ajustes à mão (`ajustes.py`)

`ajustes.py` roda as trocas de `ajustes_codigo.py`, `ajustes_cookie.py`,
`ajustes_testes.py` e `ajustes_docs.py` (registro e trocas em
`ajustes_base.py`). Cada troca exige o trecho velho exatamente onde se
espera; se o HEAD mudou o trecho, o script para e diz qual. Para refazer
um ajuste: rode o pipeline com `--ate-as-regras`, edite e atualize o
trecho no `ajustes_*.py`.

- `ajustes_codigo.py`: `src/arquivo_privado.rs` (declara `leitura`),
  `src/transporte/falha.rs` (as nove variantes saem do enum e dos três
  `match`; entram as quatro com o nome do arquivo), `std::fs` de
  `transporte.rs`, o login (`CredenciaisIlegiveis`, `OrigemIlegivel` e
  `SessaoGravadaInvalida` saem; a recusa de
  `validar_origem`/`validar_cookie` sobe como `ErroLogin::Transporte`) e
  `documento_html_privado_cli.rs` (a fonte do HTML é o arquivo).
- `ajustes_cookie.py`: o estado de `CookieDaSessao` e `CookieLido`; a
  abertura guarda o cookie em `Rede` e o entrega; `ler_cookie` sai; os
  pontos que a regra 10 não alcança porque o cookie atravessa outra
  função (`processo_sobrestar`, os dois de `processo_marcar`), a troca de
  unidade, o editor de `documento salvar` e os fatos de `ErroCriar`.
- `ajustes_testes.py`: a variante esperada depende de qual arquivo falhou
  (`tests_snapshot.rs`, `transporte/tests.rs`, `tests_envio.rs`); a troca
  de unidade recebe o cookie conferido (o caso do cookie ausente deixou
  de ser representável) e a gravação recusada é o `ErroLogin` exato;
  `tests/cli_unidade_trocar.rs` (nome em variável),
  `tests/cli_unidade_snapshot_offline.rs` (a causa antiga vira duas) e o
  módulo `support/falha_por.rs` nos binários que passaram a usá-lo.
- `ajustes_docs.py`: `contracts/cli.md`, `contracts/cli/sessao-login.md`,
  a ADR 0005, `docs/decisoes/arquivos-privados/gravacao-privada.md` e
  `docs/erros-tipados/locais.md`.

O ast-grep roda pelo `uvx` (`ast-grep-cli==0.51.2`); o `sg` do sistema
pode ser outro programa. Dentro de macro (`assert_eq!`, `matches!`) o
ast-grep só vê `token_tree`: as regras com sufixo "em macro" reescrevem o
`token_tree` mais interno que tem o caminho.
