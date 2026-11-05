---
type: ArchitectureDecision
title: Replay de cassettes fora da CLI pública
description: ies replay sai da CLI; o replay é a suíte de testes do acervo mais scripts/replay, e zip e sha1 viram dependências de desenvolvimento
status: accepted
decision: testes-do-acervo-mais-script
---

# Replay de cassettes fora da CLI pública

Decisão do proprietário de 05/11/2026: alternativa (c).

## Problema

`ies replay CENARIO` lia cassettes do acervo público e mostrava a caixa,
o histórico, documentos, versões e resultados de pesquisa gravados. Para
isso o binário de produção ligava `zip` (e, por ele, `flate2`, `zlib-rs`,
`crc32fast`, `indexmap`, `hashbrown`, `equivalent`, `getrandom` e
`typed-path`) e `sha1`, que nenhum comando do SEI usa. Desde as decisões
de replay do acervo ([processo obter](../processo-obter/replay-do-acervo.md),
[unidade](../unidade-listar-processos/replay-do-acervo.md),
[processo buscar](../processo-buscar/replay-do-acervo.md)), a conformidade
das capturas é provada por testes `#[ignore]` que passam **toda** resposta
gravada pelos leitores de produção; o subcomando ficou como uma segunda
porta, com leitores e opções próprios (filtros, seleção de passos,
consolidação, exportação) que nenhum comando de domínio tem.

O transporte ao vivo dependia de `crate::cassette::decode_http_body`
(caixa, login e autocompletar do envio): o módulo de cassettes não podia
sair do binário sem mover essa peça.

## Alternativas

- **(a) feature do cargo** (`--features replay`): o subcomando fica,
  atrás de uma feature desligada por padrão. Mantém a superfície e o
  código, duplica a matriz de build (clippy e testes com e sem a feature)
  e o contrato da CLI passa a depender de como o binário foi compilado.
  Rejeitada.
- **(b) binário no `xtask`**: o replay vira `cargo run -p xtask --bin
  replay`. O `xtask` só tem verificadores de infraestrutura e nenhuma
  dependência ([política de CI](../../ci.md)); os leitores são `pub(crate)`
  do binário `ies` e teriam de virar biblioteca. Rejeitada.
- **(c) testes do acervo mais script** (escolhida): o módulo `cassette`
  e todo replay ficam em `#[cfg(test)]`; `zip` e `sha1` vão para
  `[dev-dependencies]`; o subcomando, seus argumentos e módulos saem; o
  [script `scripts/replay`](../../../scripts/replay) lista e executa os
  testes ignorados. Custo: as opções que só a ferramenta tinha deixam de
  existir (abaixo), e quem lia uma captura avulsa passa a escrever um
  teste.

## Efeito

- `src/transporte/corpo.rs`: `decode_http_body` e `legivel`, com testes;
  nada fora de teste usa `crate::cassette`.
- `mod cassette` é `#[cfg(test)]`; `zip` e `sha1` são
  `[dev-dependencies]`. `base64` e `sha2` continuam em `[dependencies]`:
  o proxy monta `Proxy-Authorization: Basic` e o login e o anexo usam
  SHA-256.
- Saíram `replay_args`, `replay_cli`, `replay_consulta`,
  `replay_documento_cli`, `replay_pesquisa_cli`, `replay_versoes_cli`,
  `replay_busca_documento`, e o que só eles usavam: `historico_cli`,
  `filtro_andamentos`, `resumo_processo`, `unidade_lista_cli::mostrar_caixa`,
  `saida::humano`, `ErroHistorico::mensagem` e os leitores
  `cassette::read_cassette_*` com a seleção de operação e passos.
  `saida::json`, `paginas::Cobertura::completa` e
  `historico::numero_do_historico` ficaram só em `cfg(test)`, como
  oráculos dos testes.
- `ies replay …` é erro de uso do Clap (saída 2, nada em stdout):
  `replay_nao_e_mais_subcomando_da_cli` em `tests/cli_contract.rs`.
- `scripts/validar-cassettes.sh` e `scripts/validar-pesquisa-cassette.sh`
  chamavam `ies replay` e saíram; `scripts/perf_audit.py` perdeu os casos
  que mediam `ies replay` sobre cassettes sintéticas.

Medido em 05/11/2026 (Linux x86-64, `cargo build --release`):

| Medida | Antes | Depois |
|---|---:|---:|
| Linhas de `cargo tree -q -e normal --prefix none -p ies \| sort -u` | 69 | 58 |
| Crates distintos (sem os marcadores `(*)`) | 63 | 53 |
| `target/release/ies` (bytes) | 6.406.552 | 5.985.968 (−420.584, −6,6%) |

`cargo tree -e normal -i zip` e `-i sha1` não imprimem nada depois.

## Cobertura: cada opção antiga e o teste que a cobre

Os nomes de teste são do binário `ies`; os `#[ignore]` rodam com
`scripts/replay FILTRO`. "Só da ferramenta" quer dizer que nenhum comando
de domínio tinha o comportamento: ele saiu junto com o código.

| Opção de `ies replay` | Leitor | Coberto hoje por | Destino |
|---|---|---|---|
| `CENARIO` (caixa, um GET de `procedimento_controlar`) | `paginas::juntar` | `visao_do_controle::acervo::replay_do_controle_inteiro` (lista R e categoria de cada resposta gravada); golden e recusas em `saida::tests_golden` | golden portado |
| `--unidade SIGLA` na caixa | `paginas::juntar(.., Some)` | `saida::tests_golden::unidade_solicitada_precisa_ser_confirmada_pela_tela`; o acervo do Controle confere a unidade entre páginas | portado |
| `--passo N`, `--todas-paginas` | `paginas::juntar` com várias páginas | `paginas::tests` (consolidação, lacuna, repetição, faixa), sintéticos; o acervo do Controle lê cada página e seu plano | seleção de passos: só da ferramenta |
| `--numero-processo` na caixa | `resumo_processo` | nenhum leitor de produção | só da ferramenta |
| histórico (escolhido pelo conteúdo) | `historico::ler_historico` | `processo_obter::acervo::replay_do_acervo_inteiro` (leitor de andamentos de toda resposta `procedimento_consultar_historico`) e `historico::tests` | coberto |
| `--numero-processo` no histórico | número exato contra o título | `historico::tests` (`historico_aceita_protocolo_opaco_sem_mascara`, `recusa_outro_processo_e_tabela_incompleta`); controle no NUP em `tests/cli_contract.rs` | coberto |
| `--termo-andamento` | `filtro_andamentos` | — | só da ferramenta |
| `--documento [--numero-documento]` | `documento_web::identidade_do_titulo` | `processo_obter::acervo_documento::replay_de_documento_no_acervo` (título e texto de cada `documento_visualizar`); `documento_web::tests` e `documento::tests` | coberto; a operação gravada `documento_imprimir_web` não tem leitor de produção |
| `--texto-documento`, `--arquivo-texto` | `texto_do_corpo`, `arquivo_texto_documento::salvar` | o mesmo acervo de documento; `arquivo_texto_documento::tests` (0600, sem sobrescrever, links) | coberto |
| `--listar-documentos` | identidades distintas | — | só da ferramenta |
| `--termo-documento` | busca no texto capturado | — (`documento buscar` usa a pesquisa Web, coberta por `pesquisa_web::acervo`) | só da ferramenta |
| `--historico-versoes`, `--id-documento-interno`, `--versao-documento` | `documento_versoes::interpretar_historico` | `processo_obter::acervo_versoes::replay_de_versoes_no_acervo` e os testes de `documento_versoes` | coberto |
| `--resultado-pesquisa --passo N [--numero-processo]` | `pesquisa_web::ler_resultados` | `pesquisa_web::acervo::replay_do_acervo_inteiro` ("resposta do envio", via `ler_resposta_html`) e `pesquisa_web::tests` | coberto |
| `--unir-resultados-pesquisa` | consolidação de respostas | — | só da ferramenta |
| `--acervo DIR` e a conferência do card | `cassette::conferencia` | `cassette::tests` (portado: cassette íntegra, SHA-1 e SHA-256 adulterados, host fora da instalação) e cada teste do acervo | portado |

Nenhum leitor de produção ficou coberto só pelo subcomando: os portados
são o golden da caixa resumida (`tests/fixtures/caixa-v2.json`), as recusas
de `paginas::juntar` e a conferência de hash, que só os testes de
integração do replay exercitavam.

## Testes de integração `tests/replay_*.rs`

Dirigiam o binário com acervos sintéticos. Sem o subcomando, viraram:

| Arquivo | Testes | Destino e motivo |
|---|---:|---|
| `replay_unidade.rs` | 15 | 3 sem replay (origem e configuração da caixa) movidos para `tests/cli_unidade_origem.rs`; 4 (duas tabelas, golden, subcódigo do leitor, unidade) portados para `src/saida/tests_golden.rs`; hash adulterado portado para `src/cassette/tests.rs`; 2 de passos e 4 de NUP apagados (só da ferramenta) |
| `replay_historico_nup.rs` | 5 | apagados: 3 duplicam `historico::tests` e o controle no NUP de `tests/cli_contract.rs`; 2 eram do filtro de andamentos |
| `replay_documento.rs` | 6 | apagados: a identidade e o número duplicam `documento_web::tests` e `documento::tests`; a exportação duplica `arquivo_texto_documento::tests`; a escolha por número na cassette era só da ferramenta |
| `replay_lista_documentos.rs` | 2 | apagados: só da ferramenta |
| `replay_pesquisa_multipla.rs` | 2 | apagados: consolidação só da ferramenta; o leitor está em `pesquisa_web::tests` |

`tests/hex/mod.rs` só servia a esses arquivos e saiu. Também saíram os
testes unitários dos módulos removidos (`cassette/tests.rs` antigo, com a
seleção de operação e passos, `testes_nome_cenario.rs` e os de
`replay_*`, `filtro_andamentos` e `resumo_processo`).

## Verificação

- Os onze testes `#[ignore]` do acervo, por `scripts/replay acervo`
  (10 min), deram as mesmas 734 linhas de relatório que a rodada de antes
  da mudança; só mudou a linha de resumo do executor (tempo e quantos
  testes foram filtrados, já que há menos testes no binário).
- Gates do pré-push e `cargo check --target x86_64-pc-windows-gnu
  --all-targets` sem erros nem avisos.

## Rever quando

- Um comando de domínio precisar ler cassettes em produção (não deve: as
  [instruções](../../../AGENTS.md) separam captura de comando).
- Uma captura avulsa for lida com frequência e escrever o teste ficar caro:
  aí um teste ignorado parametrizado por variável de ambiente, ainda em
  `cfg(test)`, antes de qualquer subcomando.
