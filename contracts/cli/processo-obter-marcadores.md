---
type: Contract
title: Marcadores do processo pela barra da árvore
description: ies processo NUP lê a lista de marcadores do processo (nome, cor, texto, usuário, data) na tela andamento_marcador_gerenciar publicada na barra do nó do processo, por GET, sem tocar nos controles de escrita
---

# Marcadores de `ies processo NUP` (Refs #59)

Complementa o [contrato da árvore](processo-obter-arvore.md). Depois da
árvore conferida, dos andamentos e das propriedades, o comando lê os
marcadores do processo. Estrutura em
[marcadores do processo](../../docs/como-o-sei-funciona/marcadores-do-processo.md);
decisão em [processo.obter](../../docs/decisoes/processo-obter/marcadores.md).

**Um processo pode ter mais de um marcador** (decisão do proprietário,
04/11/2026; observado na UnB 4.0.9 com dois). Toda representação é
**lista**, um objeto por marcador; nomes nunca são concatenados.

## Link na barra do processo

O mesmo leitor da barra das
[propriedades](processo-obter-propriedades.md): a linha inteira
`Nos[i].acoes = '<literal>';` do nó `PROCESSO`, as tags `<a>` pelo
tokenizador de atributos, o `href` com entidades decodificadas. O link é
o único com `acao=andamento_marcador_gerenciar` (ícone "Gerenciar
Marcador", observado nas barras da UnB, de GO e da PEN1), conferido como
os outros links assinados: `controlador.php?` relativo, uma `acao`,
`id_procedimento` igual ao da árvore, um `infra_hash` de 64 hex, chaves
canônicas, mesma instalação.

**Link não publicado.** Barra ausente, vazia ou sem o link não derruba o
comando: sai `"marcadores": null`, `"marcadores"` fica em `nao_lidos` e
`"marcadores_nao_lidos_motivo": "tela_de_marcadores_nao_publicada_na_barra"`.
Link presente mas inválido ou repetido é recusa tipada (só quando os
marcadores são pedidos).

## A tela

Um GET do link (+1 requisição). `200` com
`<title>SEI - Marcadores do Processo <NUP></title>`. A tela tem
**escrita** (`btnAdicionar` → `andamento_marcador_cadastrar`,
`btnRemover` → `acaoRemocaoMultipla()`, que muda o `action` de
`form#frmGerenciarMarcador` para `andamento_marcador_remover`, e por linha
`andamento_marcador_alterar` e `acaoRemover`): **nada** disso é enviado,
clicado ou seguido; o comando só faz o GET. Leitura pela árvore do `tl` e
pelo tokenizador de atributos, sem regex e sem JavaScript:

- **Identidade**, antes de qualquer linha: o `title` único começa por
  `SEI - Marcadores do Processo ` e o resto é o NUP pedido (comparação
  opaca); o único `form#frmGerenciarMarcador` tem `action` com
  `acao=andamento_marcador_gerenciar` e um `id_procedimento` igual ao id
  da árvore.
- **Tabela**: uma `table#tblMarcadores`, sem tabela aninhada. A legenda é
  `Marcadores (N registro):` ou `Marcadores (N registros):`, sem faixa. O
  cabeçalho é exatamente: seleção (`lnkInfraCheck`), `Marcador`, `Texto`,
  `Usuário`, `Data/Hora`, `Ações` (rótulo de
  `div.infraDivRotuloOrdenacao` ou do `th`).
- **Cada linha** (`tr` com `td`, uma célula por coluna) é um marcador:
  - `id`: `value` decimal do `input.infraCheckbox`; o `a#lnkInfraID-<id>`
    e o `td#tdAndamentoMarcador<id>`, quando presentes, têm o mesmo id.
  - `nome`: o texto da célula Marcador (espaços e `&nbsp;` colapsados);
    o `title` do ícone e o do checkbox, quando presentes, são iguais a ele.
  - `cor`: de `svg/marcador_<cor>.svg` (um `img`, `<cor>` em minúsculas
    ASCII e `_`).
  - `texto`: o texto de `td#tdAndamentoMarcador<id>`; vazio é `null`.
  - `usuario`: o texto de `a.ancoraSigla`; célula vazia é `null`. O nome
    completo (`title`) não é publicado.
  - `data_hora`: `dd/mm/aaaa hh:mm`.
  - Ações: nunca lidas nem seguidas.
- Id repetido é recusa. Mais linhas que a legenda é recusa; **menos**
  publica `marcadores_total_declarado` e `marcadores_faixa_observada` e
  não alega completude (a tela não mostrou paginação; não é pedida outra
  página).

**Tela sem tabela.** A tela deste processo sem `table#tblMarcadores`
(forma provável de zero marcadores, **não observada**): sai
`"marcadores": null`, `"marcadores"` fica em `nao_lidos` e o motivo é
`"tabela_de_marcadores_nao_publicada_na_tela"`. Nunca `[]` sem
observação.

**Escopo da unidade.** O link e a tela são assinados para a unidade da
sessão (`infra_unidade_atual`). Que a tela mostra só os marcadores dessa
unidade é **inferência** (o SEI cadastra marcadores por unidade; a tela
observada mostrou os mesmos dois marcadores da célula do Controle da
mesma unidade); marcadores de outras unidades não foram observados.

## Índice local

O link lido é guardado (`acao` `andamento_marcador_gerenciar`, objeto =
id do processo). No **caminho 1** (árvore pelo link guardado) o guardado
é tentado primeiro, só se a barra recém-lida oferece a tela; login,
redirect ou página que não é a tela deste processo o apagam, e o link da
barra é usado. Status de falha é erro de transporte, sem nova tentativa.

## Orçamento

Sequências exatas (testes `processo_obter::tests_marcadores`):

| Caminho | Padrão | `--sem-marcadores` | `--sem-andamentos --sem-propriedades` | os três |
| --- | --- | --- | --- | --- |
| 1. guardados (árvore, histórico, propriedades, marcadores) | 4 | 3 | 2 | 1 |
| 1. marcadores guardados velhos | 5 | 3 | 3 | 1 |
| 4a. linha do Controle | 6 | 5 | 4 | 3 |

Ordem dos GETs depois da árvore: histórico, propriedades, marcadores.
Sem link na barra, os marcadores custam 0. Nos outros caminhos, +1 sobre
o [contrato das propriedades](processo-obter-propriedades.md).

## Saída

```json
"marcadores": [
  {"id": "501", "nome": "texto", "cor": "azul", "texto": null,
   "usuario": "sigla", "data_hora": "dd/mm/aaaa hh:mm"},
  {"id": "502", "nome": "texto", "cor": "amarelo", "texto": "texto",
   "usuario": "sigla", "data_hora": "dd/mm/aaaa hh:mm"}],
"marcadores_total_declarado": 2,
"marcadores_faixa_observada": {"inicio": 1, "fim": 2}
```

Lidos, `"marcadores"` sai de `nao_lidos`; a faixa é `null` com zero
linhas. `--sem-marcadores`: nenhum pedido; as chaves `marcadores*` somem,
`"marcadores"` fica em `nao_lidos` e o motivo é
`"omitidos_por_sem_marcadores"`. Texto humano: uma linha por marcador
(`- <nome> [<cor>]: <texto>`).

## Efeitos

GET de tela de gerenciamento: declarado **leitura** (nenhuma mudança de
estado observada). Nunca POST, nunca `btnRemover`/`btnAdicionar`. Nada
a restaurar.

## Falhas (saída 1)

Link: `barra_do_processo_fora_da_forma_observada`,
`barras_do_processo_repetidas_na_arvore`,
`links_dos_marcadores_repetidos_na_barra`,
`endereco_dos_marcadores_nao_reconhecido`,
`endereco_dos_marcadores_de_outro_processo`.

Tela: `html_dos_marcadores_nao_interpretado`,
`tela_dos_marcadores_ausente`,
`numero_dos_marcadores_diferente_do_solicitado`,
`id_dos_marcadores_diferente_do_aberto`,
`tabelas_dos_marcadores_repetidas`,
`legenda_dos_marcadores_fora_da_forma_observada`,
`cabecalho_dos_marcadores_nao_reconhecido`,
`linha_do_marcador_fora_da_forma_observada`,
`marcador_repetido_na_tela`, `marcadores_alem_do_total_declarado`.

Login fora de palpite: `sessao_encerrada_rode_ies_login`; 302 de outra
forma: `redirecionamento_nao_reconhecido`; status HTTP de falha, o erro
de transporte com `status_http`, sem retry.

Validação live: **NOT_RUN** (sem execução do comando nesta fatia; o
agente pai valida na UnB).
