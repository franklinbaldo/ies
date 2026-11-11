---
type: Contract
title: Listar processos na visão detalhada
description: ies unidade listar-processos [--pagina N] lê a visão D, publica colunas e categoria observada e deixa a sessão na D, sem restaurar
---

# `ies unidade listar-processos` na visão detalhada (Refs #57)

Complementa o [contrato de `unidade.listar_processos`](unidade-listar-processos.md).
Decisão do proprietário (02/11/2026): o comando lê a **visão detalhada**
(`hdnTipoVisualizacao=D`, tabela `tblProcessosDetalhado`), que publica numa
lista só os processos de Recebidos e Gerados (total = soma das duas) e as
colunas configuradas no [Nível de detalhe](unidade-colunas.md). As listas
com categoria completa continuam em `ies unidade recebidos|gerados`
([listas](unidade-recebidos-gerados.md)).

## Saída

Um **array com uma lista** na [forma única](unidade-recebidos-gerados.md),
mais `colunas_ausentes`:

```json
[{"unidade": "TESTE", "origem": "web",
  "total_declarado": 332, "faixa_observada": {"inicio": 1, "fim": 100},
  "colunas_ausentes": ["anotacao", "especificacao", "prioridade",
                       "observacao", "ultima_movimentacao", "marcadores"],
  "processos": [{"id": "42", "numero": "000042/2026", "visualizado": true,
                 "sigiloso": false, "categoria": "recebidos",
                 "colunas": {"atribuicao": "fulano", "tipo": "Ofício",
                             "interessados": ["Interessado A", "Interessado B"],
                             "marcadores": [
                               {"nome": "Pendente", "cor": "amarelo",
                                "texto": "texto do marcador", "id": "31",
                                "rotulo": "31#Pendente"},
                               {"nome": "Revisão", "cor": "azul",
                                "id": "32",
                                "rotulo": "32#Revisão"}]}}]}]
```

- A **lista** não tem campo `categoria`: D é modo de exibição, não categoria.
- `categoria` do **processo** é `"recebidos"` ou `"gerados"` quando o `id`
  apareceu nas tabelas `tblProcessosRecebidos`/`tblProcessosGerados` da
  **mesma** tela R obtida pelo GET deste comando; senão a chave é
  omitida: a D lida sozinha não produz a categoria (#749). A D não traz marcador de
  categoria por linha (verificado nas capturas de UnB, PEN1 e GO); a
  informação só existe porque a R foi lida antes, sem requisição a mais.
  Sessão já na D: nenhum processo traz `categoria`. Como o comando deixa a sessão na D, as
  leituras seguintes (PEN1, GO) saem sem `categoria` até algo voltar à R; as
  listas por categoria são `ies unidade recebidos|gerados`.
- `colunas`: objeto com as chaves estáveis das colunas **presentes no
  cabeçalho** (`atribuicao`, `anotacao`, `tipo`,
  `especificacao`, `prioridade`, `interessados`, `observacao`,
  `controle_de_prazo`, `para_devolver`, `aguardando_retorno`,
  `ultima_movimentacao`, `marcadores`; ordem alfabética no JSON). Texto da célula com espaços
  colapsados (inclusive `&nbsp;`); célula de texto vazia tem a chave omitida. `interessados` é
  um **array** com o texto de cada `div.divRotuloItemCelula` (itens
  separados pelo HTML, PEN1); célula vazia é `[]`. `marcadores` é um
  **array** de objetos, um por marcador (um processo pode ter vários):
  ver [Marcadores](#marcadores). As demais colunas são texto único.
  Colunas sem rótulo (seleção, ícones, ações) não entram.
- `colunas_ausentes`: chaves das **nove** colunas configuráveis de
  cabeçalho de texto do catálogo que não vieram no cabeçalho; `[]` quando
  todas estão presentes (máximo = 9 + Processo, decisão do proprietário).
  Sem o GET da configuração (que este comando não faz), não diz se a
  coluna **existe** na instalação: o catálogo do Nível de detalhe varia
  por versão (a UnB 4.0.9 não publica Prioridade), então uma chave
  ausente pode não ser marcável ali. `ies unidade colunas` publica a
  disponibilidade (`disponivel`) de cada opção.
  Prazo, devolver e aguardando têm cabeçalho só de ícone e não contam
  aqui. A saída humana sugere marcá-las no Nível de detalhe; `ies unidade
  colunas` mostra a configuração (não há comando de escrita).

### Marcadores

Forma observada na UnB 4.0.9 (04/11/2026, todas as colunas do Nível de
detalhe marcadas): a célula tem **uma** `a.ancMarcador` **por marcador**,
`<a class="ancMarcador" href="controlador.php?acao=andamento_marcador_gerenciar&…&id_procedimento=<id>&…#ID-<n>"
onmouseover="return infraTooltipMostrar('<texto>');" …><div><img src="svg/marcador_<cor>.svg?…" class="imagemStatus"/></div><div><nome></div></a>`;
sem marcador, `&nbsp;`. Cada âncora vira
`{"nome", "cor", "texto", "id"}`, na ordem servida:

- `nome`: texto do segundo `div` (entidades decodificadas, espaços
  colapsados), não vazio.
- `cor`: `<cor>` de `svg/marcador_<cor>.svg` (minúsculas ASCII e `_`),
  único `img` do primeiro `div`.
- `texto`: o literal JS de aspas simples do `onmouseover`, na forma exata
  `return infraTooltipMostrar('<literal>');` (escapes `\'`, `\"`, `\\`,
  `\/`, `\n`, `\r`, `\t`; nenhum JavaScript executado), espaços
  colapsados; vazio ou sem `onmouseover`: chave omitida.
- `id`: `<n>` do fragmento `#ID-<n>` (decimal; o mesmo id da linha da
  tela de marcadores do processo); sem fragmento: chave omitida. É o
  `id_marcador` do **marcador da unidade**, não da atribuição ao processo:
  na UnB 4.0.9 o mesmo `#ID-<n>` aparece em linhas de processos
  diferentes e é o `value` do checkbox e o `id_marcador` do link de
  alterar na tela de marcadores.
- `rotulo`: `<id>#<nome>` (por exemplo `31#Pendente`), para o agente ler
  o nome e citar o marcador sem ambiguidade; o id é o trecho decimal antes
  do primeiro `#` (o nome pode conter `#`). Sem `id`, chave omitida: o nome
  sozinho não identifica.
- O `href` precisa ser `controlador.php?` com uma `acao` igual a
  `andamento_marcador_gerenciar` e `id_procedimento` igual ao da linha. O
  link não é seguido nem publicado.

Célula vazia é `[]`. Texto ou tag fora das âncoras é recusa
(`texto_fora_dos_marcadores_na_celula_detalhada`); âncora fora dessa
forma, recusa (`marcador_da_celula_detalhada_fora_da_forma_observada`).
Só a UnB tinha marcadores nas capturas; GO, PEN1 e PEN4 sem marcador na
D capturada, então outra forma é recusa até ser observada.

**Mudança incompatível (04/11/2026).** Até esta versão `marcadores` saía
como **texto único**, e uma célula com dois marcadores publicava os
nomes colados numa string só (medido ao vivo na UnB 4.0.9).
Agora é **array** de objetos; vazio era `null` e passa a `[]`. Quem lia a
string precisa ler a lista. As demais chaves e o arquivo de `--saida`
não mudam; `ies unidade
recebidos|gerados` (visão R) não publica colunas e não muda. A colheita do
Controle no índice usa a mesma leitura e as mesmas recusas: com âncoras
na forma observada colhe os links como antes.

Recusas da tabela D, além das do contrato da operação:

| Fato observado | Erro (stderr) |
| --- | --- |
| Cabeçalho com rótulo fora de Processo e das 12 colunas | `ErroDetalhada::CabecalhoDaVisaoDetalhadaNaoReconhecido` |
| Mesma coluna duas vezes no cabeçalho | `ErroDetalhada::ColunaDaVisaoDetalhadaRepetida` |
| `data-label` da célula diferente do cabeçalho da coluna | `ErroDetalhada::RotuloDaCelulaDiferenteDoCabecalho` |
| Interessados com texto fora de `divRotuloItemCelula` | `ErroDetalhada::ItensDaCelulaNaoSeparados` |
| Itens `divItemCelula` em coluna de texto único | `ErroDetalhada::ItensNaoPrevistosNaCelulaDetalhada` |
| Marcadores com texto ou tag fora de `a.ancMarcador` | `ErroDetalhada::TextoForaDosMarcadoresNaCelula` |
| `a` da célula Marcadores fora da forma observada | `ErroDetalhada::MarcadorDaCelulaDetalhadaForaDaFormaObservada` |

Rótulo do cabeçalho: texto de `div.infraDivRotuloOrdenacao` (ou do `th`);
sem texto, o `aria-label` do ícone (SEI 5.0.x) ou o argumento de
`infraTooltipMostrar('…')` (UnB 4.0.9). Cabeçalho sem rótulo fora de
`infraDivRotuloOrdenacao` é coluna sem rótulo; com o `div` e sem rótulo,
recusa.

## Sequência e orçamento: a tela fica onde o comando a deixou

O comando **não restaura** a visão nem a página que moveu (decisão do
proprietário, 02/11/2026, na
[decisão da operação](../../docs/decisoes/unidade-listar-processos.md)):
vindo da R, a sessão fica na D; com `--pagina N`, a D fica lembrando a
página N. Toda saída publica `faixa_observada`, então a página lembrada é
visível na próxima leitura. Requisições, sem retry:

| Tela no GET | `--pagina` | Requisições |
| --- | --- | --- |
| D | ausente ou igual à página exibida | 1 GET |
| D | N diferente da página exibida | 1 GET + 1 POST (página N) |
| R | ausente ou igual à página lembrada da D | 1 GET + 1 POST (D) |
| R | N diferente da página lembrada da D | 1 GET + 2 POST (D, página N) |

Regime estável: depois da primeira troca a sessão está na D (PEN1, PEN3,
PEN4 e GO lembram a D; a UnB 4.0.9 volta à R no GET seguinte), e
`listar-processos` custa **1 GET**.

1. **GET** do Controle (travas existentes). Visão nem R nem D:
   `visao_do_controle_nao_reconhecida`.
2. **R:** lê as tabelas R (mesmas recusas da caixa, unidade obrigatória).
   `--pagina N` com N maior que `max(1, recebidos + gerados)` declarados é
   `pagina_da_caixa_fora_do_intervalo` **sem POST** (limite seguro: o tamanho da
   página da D só é conhecido depois de vê-la). **POST D** derivado de
   `frmProcedimentoControlar`, como `trocarVisualizacao('D')`: só
   `hdnTipoVisualizacao` muda. A resposta precisa ser D, da mesma unidade.
3. A D mostra a **sua** página lembrada (independente das páginas da R).
   Tamanho e número de páginas vêm de `hdnDetalhadoPaginaAtual` e da faixa,
   como em `--pagina N` da R. N fora do intervalo: `pagina_da_caixa_fora_do_intervalo`
   antes do POST de página (vindo da R, já depois do POST D; a sessão fica
   na D). D sem `hdnDetalhadoPaginaAtual` (UnB 4.0.9, sem paginação) só
   aceita a página 1, e só se a faixa cobre o total.
4. **POST página N** a partir do formulário da D, com
   `hdnDetalhadoPaginaAtual = N-1` (a R não publica esse campo; não é
   inventado). A resposta precisa manter unidade, visão D, total e
   `hdnDetalhadoHashCriterios`, e trazer exatamente a faixa da página N.
   N igual à página que a D mostra não faz POST de página.

Uma falha em qualquer passo é publicada como está, sem lista e sem
requisição a mais; a tela fica onde o último POST aceito a deixou (vindo
da R, na D, se o POST D passou). Se o próprio POST D ou o de página falha,
a resposta não comprova onde a sessão ficou: o próximo GET mostra.

Histórico: até 02/11/2026 o comando restaurava a R (e a página encontrada
da D) depois de ler, com até 1 GET + 4 POST e as recusas
`visao_do_controle_nao_restaurada` e
`pagina_da_visao_detalhada_nao_restaurada`, que deixaram de existir.

## Demais recusas da troca

| Fato | Erro (stderr) |
| --- | --- |
| 302 ao trocar de visão | `ErroVisaoDoControle::RedirecionamentoAoTrocarVisao` |
| Resposta da troca em outra visão | `ErroVisaoDoControle::VisaoNaoTrocadaAposEnvio` |
| Unidade diferente na resposta da troca | `ErroVisaoDoControle::UnidadeAlteradaAoTrocarVisao` |
| `hdnDetalhadoPaginaAtual` ilegível ou repetido | `PaginaAtualDaVisaoDetalhadaIlegivel` |
| Índice incompatível com a faixa | `PaginaAtualDaVisaoDetalhadaIncompativelComAFaixa` |
| Mais de uma página sem `lnkDetalhado*`/`selDetalhadoPaginacao*` | `PaginadorDaVisaoDetalhadaNaoPublicado` |
| Critério da D diferente após paginar | `CriteriosDaVisaoDetalhadaAlteradosAposPaginar` |
| Total ou faixa diferentes após paginar | `TotalDeclaradoAlteradoAposPaginar`, `ErroVisaoDoControle::FaixaDaRespostaDiferenteDaPaginaSolicitada` |

Derivação, envio, login, MIME, charset e status seguem a
[submissão derivada](../../docs/arquitetura/0002-submeter-formulario-observado.md)
e o [transporte](../cli.md). Efeitos avaliados, não desfeitos:
`AlteraEstadoDaSessao::PreferenciaDeVisualizacao` (troca; na PEN1 um login
novo da mesma conta abriu na D, efeito cosmético) e
`PaginaOuFiltroLembrado` (página da D, visível em `faixa_observada`).

## Opções

`--saida ARQUIVO` exporta o mesmo array, depois das verificações
(categoria por processo preservada). `--pagina-arquivo` foi
**removida** (erro de uso, saída 2): unia capturas privadas sem POST e não
tem semântica junto da troca de visão, cujas telas são de outra geração.
Capturas do Controle passam pelos mesmos leitores no replay do acervo
`visao_do_controle::acervo` (`scripts/replay`; não há `ies replay`).

Golden: [`caixa-detalhada-v2.json`](../../tests/fixtures/caixa-detalhada-v2.json) (chaves em ordem alfabética).
Ensaio offline com HTML sintético modelado nas capturas de UnB, PEN1 e GO,
e leitura estrutural das próprias capturas privadas (sem copiar valores):
D da UnB (59 linhas, ícones pelo tooltip), PEN1 (páginas 1 e 2, Interessados
até 6 itens) e GO (12 colunas, `colunas_ausentes: []`); derivação da troca
nos formulários R e D das três.
Ensaio live de 02/11/2026 (PEN1, UnB, GO; [validação](../../docs/validacao-live/2026-11-02-leitura-controle.md)):
troca, leitura e restauração da visão PASS, com a versão que restaurava.
Fluxo sem restauração acima ([ensaio de 04/11/2026](../../docs/validacao-live/2026-11-04-unidade-paginacao.md)): **PASS** na
UnB 4.0.9 (R→D, 1 GET + 1 POST; a UnB volta à R a cada GET) e na PEN1,
PEN3 e PEN4 5.0.4 (D com 3 páginas, lembra a D), nos quatro
regimes da tabela, com as requisições contadas e a página da D lembrada.
UFRJ 4.1.5: D de uma página com `hdnDetalhadoPaginaAtual=0`, sem
paginador e legenda sem intervalo; recusada na rodada e **PASS** depois
da correção (o índice `0` sozinho não é paginação). GO 5.0.3 **PASS** na
terceira tentativa (R→D e D lembrada); PEN2 **NOT_RUN** ([GO e PEN2](../../docs/validacao-live/2026-11-04-unidade-paginacao-go-pen2.md)).

## Histórico de restauração

As recusas de restauração citadas neste contrato são históricas: foram
aposentadas pela decisão de #57 de deixar a sessão na visão/página
utilizada, sem requisições para restaurar a tela. Não são emissões ativas.
