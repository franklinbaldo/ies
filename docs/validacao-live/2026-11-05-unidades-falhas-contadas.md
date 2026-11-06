---
type: SEIObservation
title: Contagem HTTP nas recusas da lupa de unidades
description: Issue 244; regressao sintética com contagem em falhas sem nova requisicao real ao SEI
---

# #244 — contagem das falhas Web (05/11/2026)

## Observação anterior ao RED

Os ensaios autenticados existentes mostram UnB recusando
`link_da_lupa_ambiguo` e GO recusando
`formulario_da_lupa_inesperado`, sem publicar a quantidade de
requisições feitas pela operação. Ver
[relatório precedente](2026-11-05-unidades-paginacao-integral.md),
[ensaio UnB 42143302555](https://github.com/franklinbaldo/ies/actions/runs/42143302555)
e [ensaio GO 42143615203](https://github.com/franklinbaldo/ies/actions/runs/42143615203).
Nenhuma captura comprova qual variação de HTML causou as recusas;
por isso não relaxamos validação de links ou formulários.

## Contrato e teste sintético

A coleta passa um contador mutável, inicializado em zero, pelos
GETs e POSTs da lupa. A tentativa é contada antes de chamar o
transporte. O mesmo contador é preservado em todas as recusas, sem
trocar o subcódigo de erro, expor cookie, URL assinada ou
`infra_hash`, ou repetir tentativa.

O shim inicial do commit `131578e` sempre reportava zero em
caso de falha. Dois testes novos, publicados em `4e83286`,
exigem (a) **5 requisições** na falha do primeiro POST
(4 GET + 1 POST) e (b) **278 requisições** na falha do último
POST da coleta sintética de 274 páginas (4 GET + 274 POST).
Os testes não usam rede. Teste da composição JSON verifica
`requisicoes` e ausência de campos privados.

**RED:** execução isolada não confirmada; o CI inicial desse head
foi cancelado por commits subsequentes. Não atribuir contagem
de falhas de teste ao RED sem log. **GREEN:** conferir no
CI final da PR #303; não antecipar PASS.

## Validação live desta fatia

| Etapa | Resultado | Requisições |
| --- | --- | --- |
| UnB — nova coleta | NOT_RUN | 0 |
| GO — nova coleta | NOT_RUN | 0 |
| Pesquisa offline pós-coleta | NOT_RUN | 0 |
| Escrita de domínio no SEI | NOT_RUN | 0 |

Nenhuma requisição ao SEI foi feita nesta fatia. Não repetir
as chamadas falhadas dos ensaios anteriores apenas para colher a
contagem. Não há novo resíduo de processo, documento ou unidade.
A cassette GO já tinha um replay curto obrigatório no CI. Na
continuação de 06/11, a cassette **integral** foi conferida e incorporada
ao replay obrigatório, sem requisições ao SEI.

## Pendente na issue

Resolver as diferenças estruturais reais de UnB e GO com evidência,
obter a lista integral, pesquisar offline com zero HTTP e capturar
stdout/stderr real da CLI de sugestões. A contagem nas falhas
não é prova de sucesso da listagem nem conclui a issue.

## Continuação: cassette integral GO (06/11/2026)

A cassette `unidade-destinos-no-indice-go--85f61d670bfc.md`
documenta **278 interações**: 4 GET e 274 POST, com 13.671
identidades. O ZIP público `lote-5b6af062ae69d8c7.zip`
tem SHA-1 conferido `5b6af062ae69d8c7db0293c93ce30342c0aa364a`.
O novo replay Rust leu **274 páginas reais** pelo leitor de produção,
validou cada filtro e reuniu **13.671 IDs sem duplicatas**, sem
acesso HTTP. A derivação do formulário também é exercitada sobre
as 275 respostas da lupa.

**RED observado** no
[CI 42161586924](https://github.com/franklinbaldo/ies/actions/runs/42161586924):
o replay das 274 páginas passou, mas a derivação recusou o primeiro
formulário (passo 3). O diagnóstico saneado apontou
`botao_de_envio_ausente_no_formulario`. A forma real possui **dois**
`input type=submit id=btnPesquisar` **sem atributo `name`**.
A implementação anterior esperava um submit com `name=btnPesquisar`,
como nas fixtures sintéticas. O DOM de GO não tinha esse campo,
sem que isso significasse ausência do botão.

A correção distingue, dentro da operação da lupa, exatamente um
submit **nomeado** dos **um ou dois submits sem nome** publicados
sob `id=btnPesquisar`. Quando não há `name`, o corpo omite o
par de botão, como a submissão HTML normal; os campos observados
permanecem. Não executa JavaScript, não inventa links e não cria
requisições. Casos sem botão identificável, duplicados nomeados ou
mais de dois botões sem nome recusam antes do POST.
[GREEN do replay integral GO no CI 42162187792](https://github.com/franklinbaldo/ies/actions/runs/42162187792):
três testes de cassette passaram, incluindo a derivação de todos
os 275 formulários.

**Live após correção:** UnB NOT_RUN (0), GO NOT_RUN (0),
pesquisa offline NOT_RUN (0); escrita de domínio 0, resíduos 0.
A falha anterior da GO permanece registrada como FAIL histórico;
o replay offline não a reclassifica como PASS live. A incompatibilidade
de links da UnB ainda não está corrigida. O teste da CLI real para
sugestões também permanece pendente.

## Prova de CLI real com índice local sintético (06/11/2026)

O teste de integração `tests/cli_unidade_snapshot_offline.rs` executa
**o binário real** `ies` como subprocesso (não chama funções internas).
Prepara os dois arquivos privados de origem e Controle com modos `0600`,
diretório `0700`, e a tabela pública SQLite versão 3 com três unidades.
A sessão é inteiramente sintética e **não possui arquivo de cookie**:
qualquer tentativa de consultar o SEI não conseguiria autenticar-se.

O teste exige `unidade listar --json` com `origem=indice`, três
unidades, `lida_em=1234`, `requisicoes=0`; depois
`unidade pesquisar sao/cde --json`, que encontra `SÃO/CDE` sem
HTTP, e consulta direta SQLite em modo somente leitura, sem alteração
de esquema nem de linhas. Confere também que nenhuma saída contém
`infra_hash`, `PHPSESSID` ou `controlador.php`.
Com `--atualizar` e cookie ausente, exige **recusa com zero
requisições** e sem fazer o cache antigo parecer leitura nova.

**RED novo:** não observado — o teste foi acrescentado à capacidade já
implementada, e a primeira execução da suíte Rust o aprovou. O primeiro
gate `prek` desse acréscimo falhou apenas em Rustfmt/Clippy;
a correção preservou os asserts e não desabilitou testes.
**Live desta sessão:** UnB NOT_RUN (0 requisições), GO NOT_RUN (0),
pesquisa pós-coleta autenticada NOT_RUN (0), escrita NOT_RUN (0),
resíduos nenhum. A prova sintética não substitui o ensaio autenticado
nem corrige retroativamente os FAIL históricos.
