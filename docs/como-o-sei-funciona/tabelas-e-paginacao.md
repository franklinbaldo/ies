---
type: SEIObservation
title: Tabelas e paginação
description: Cobertura, linhas, cabeçalhos e estados de listagem
---

# Tabelas e paginação

Listas de acompanhamento, blocos, textos padrão e históricos podem
paginar. Foram observadas páginas de 50 ou 100 linhas. A legenda
pode declarar um total maior do que o número de linhas presentes.
O servidor também pode lembrar a última página/filtro da sessão.

Transições da lista podem usar `infraAcaoPaginar` e
`hdnInfraPaginaAtual`. Algumas telas contêm mais de uma
listagem com paginação independente.

O IES agora compara o rótulo da unidade em `lnkInfraUnidade`, o
modo `hdnTipoVisualizacao` e os hashes de critérios
`hdnRecebidosHashCriterios`/`hdnGeradosHashCriterios`.
Esses últimos são marcadores opacos da página, não autorização,
identidade interna ou comprovante criptográfico de sessão.
Uma comparação não substitui a verificação de faixas e IDs.
Sem contexto suficiente, não juntar páginas offline.

[Relato Todos #1200](https://github.com/franklinbaldo/todos/issues/1200):
na instalação PGE-IPERON/RO, a caixa resumida paginou acima de 500
processos e tinha paginação independente de Recebidos/Gerados.
O relato recomenda testar também a visão detalhada, mas não autoriza
alterar automaticamente a preferência visual do usuário no IES.

A estrutura não é uniforme: há cabeçalhos `colspan`, tabelas
que reutilizam `id` em contextos diferentes, células sem nome e
linhas de comentário adicionais. Ler por cabeçalho e identidade
comprovada é mais seguro que presumir posição absoluta.

Textos importantes podem estar em atributos `title` ou
`aria-label`: nome de unidade, usuário, tipo de processo,
anotação, sinalizações e ações disponíveis na linha.

**Nunca transformar parcial em completo:** conferir legenda,
total de registros, intervalos, número de linhas e transições.
Ausência de cursor não prova universalmente fim dos dados.
Uma listagem incompleta/indeterminada precisa informar isso.

## Classes do link de processo no Controle

SEI Goiás 5.0.3 treinamento, 02/11/2026: link de processo sigiloso
com classe `processoVisualizadoSigiloso` em `tblProcessosGerados`,
ao lado de `processoVisualizado` nas demais linhas. O acervo de
Goiás já tinha a classe em `tblAcompanhamentos`. O par
`processoNaoVisualizadoSigiloso` decorre da nomenclatura e não foi
observado. O número de processo em Goiás tem 15 dígitos sem pontuação
e permanece opaco.

No acervo arquivado de Goiás, a visão resumida pagina com
`lnkRecebidosProximaPagina*`, `selRecebidosPaginacao*` e
`hdnRecebidosPaginaAtual` (idem Gerados), não `lnkInfra*`; a legenda `(N registros - 1 a 10):` declara a faixa.

## Paginadores por tabela no Controle (02/11/2026)

Capturas autenticadas de treinamento, inspecionadas só pela estrutura:
PEN1, PEN3 e PEN4 (SEI 5.0.4) e Goiás (5.0.3), visão resumida.

- Cada tabela tem seu paginador em `div{T}AreaPaginacaoSuperior` e
  `div{T}AreaPaginacaoInferior` (`T` = `Recebidos` ou `Gerados`):
  `lnk{T}ProximaPagina*` chama `infraAcaoPaginar('+',0,'T', null)`;
  `lnk{T}UltimaPagina*` chama `infraAcaoPaginar('=',k,'T', null)`;
  `sel{T}Paginacao*` (com `name`) chama `infraAcaoPaginar('=',this.value,'T', null)`.
  Com duas páginas, só o link "próxima" aparece (PEN1 Recebidos, PEN3, PEN4);
  o seletor e o link "última" surgem a partir de três (PEN1 Gerados, Goiás).
- O índice começa em 0: em Goiás, 7990 registros em páginas de 10 têm
  último índice 798; 131 têm 13. As opções do seletor vão de `0` a `k`.
- `hdn{T}PaginaAtual` (valor `0` na primeira página) e
  `hdn{T}HashCriterios` ficam dentro de `frmProcedimentoControlar`, que
  é `method="post"`, `action` assinado de `procedimento_controlar` e não
  tem botão de envio. UnB e UFRJ, sem paginação, publicam os mesmos
  campos e áreas vazias: o campo não indica paginação.
- Outros controles nomeados do formulário: `txtBloco`, caixas
  `chk{T}Item*` desmarcadas e ocultos `hdn{T}Itens*`, `hdnInfraSelecoes`,
  `hdnExibir*`, `hdnMeusProcessos`, `hdnIdBloco`, `hdnIdMarcador*`,
  `hdnIdTipoProcedimento*`, `hdnIdTipoPrioridade*`, `hdnFlagControleProcessos`.
  `hdnInfraPrefixoCookie` fica fora do formulário em PEN1, PEN3 e Goiás e
  dentro em PEN4.
- Legenda: `Processos recebidos (N registros - a a b):`; páginas de 100
  em PEN e de 10 em Goiás.

O `InfraUtil.js`, que define `infraAcaoPaginar`, não foi capturado; o
efeito "grava o campo da página e envia o formulário" vem do oráculo
Todos para listas comuns. O IES lê a página que a tela mostra e só
envia um POST com `ies unidade recebidos|gerados --pagina N`
([contrato](../../contracts/cli/unidade-listar-processos-paginas.md)).

## Visão detalhada do Controle (02/11/2026, treinamento)

Observações estruturais de UnB 4.0.9, PEN1 5.0.4 e GO 5.0.3, sem valores
([ensaio](../validacao-live/2026-11-02-leitura-controle.md)):

- `hdnTipoVisualizacao` alterna `R`/`D` pelo mesmo `frmProcedimentoControlar`
  (`trocarVisualizacao`). A D tem uma tabela, `tblProcessosDetalhado`, com
  total = Recebidos + Gerados e **nenhum** marcador de categoria por linha.
- A D tem página lembrada própria (`hdnDetalhadoPaginaAtual`, paginador
  `lnkDetalhado*`/`selDetalhadoPaginacao*`), do tamanho da página da R
  (PEN1 100, GO 10). A UnB publica a D inteira, sem o campo. A R não publica
  `hdnDetalhadoPaginaAtual`.
- PEN1 e GO lembram a D na sessão (na PEN1, até num login novo); a UnB
  volta à R no GET seguinte.
- Colunas configuráveis pelo Nível de detalhe (`procedimento_configurar_detalhe`,
  link `ancNivelDetalhe` só na D): rótulo em `div.infraDivRotuloOrdenacao`;
  prazo, devolver e aguardando têm cabeçalho só de ícone (`aria-label` na
  5.0.x, `infraTooltipMostrar` na 4.0.9). Células trazem `data-label` igual
  ao cabeçalho na 5.0.x; a 4.0.9 não. Interessados separa itens em
  `div.divItemCelula`/`div.divRotuloItemCelula`. Cabeçalhos usam `&nbsp;`
  (`Última&nbsp;Mov.`).
- Colunas sem rótulo: seleção (`lnkInfraCheck`), ícones de estado e ações.
- O **catálogo do Nível de detalhe varia por versão**: PEN1 5.0.4 e GO
  5.0.3 publicam 12 caixas `chkSin*`; a UnB 4.0.9 publica 11, sem
  `chkSinNivelPrioritarios` (Prioridade). Ensaio live de 02/11/2026.
- A página lembrada da D é estado da sessão independente da visão exibida:
  na PEN1, paginar a D vinda da R e voltar à R deixou a D na página pedida,
  e a D seguinte abriu nela (ensaio live de 02/11/2026).
