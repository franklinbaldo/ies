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

## Pendente na issue (registrado antes dos ensaios de 06/11)

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

## Ensaios autenticados em treinamento — 06/11/2026

A [execução 42180230873](https://github.com/franklinbaldo/ies/actions/runs/42180230873)
rodou **o binário real** em UnB e GO, em jobs independentes,
somente leitura. Os números abaixo saíram dos eventos JSON do
comando no log, não só da conclusão verde do workflow.

| Instalação e operação | Live | HTTP da operação | Resultado |
| --- | --- | ---: | --- |
| UnB: `login --json` | PASS | 5 | sessão de treinamento |
| UnB: `unidade listar --atualizar --json` | PASS | 5 | `origem=sei`, 14 unidades, snapshot persistido |
| UnB: `unidade pesquisar --json` | PASS | 0 | `origem=indice`, 14 unidades |
| GO: `login --json` | PASS | 6 | sessão de treinamento |
| GO: `unidade listar --atualizar --json` | PASS | 278 | `origem=sei`, 13.671 unidades, snapshot persistido |
| GO: `unidade pesquisar --json` | PASS | 0 | `origem=indice`, 13.453 resultados filtrados |

Em GO, a coleta foi **4 GET + 274 POST = 278**, sem repetir
páginas. O resultado de 13.453 na pesquisa representa um **filtro**
sobre as 13.671 unidades persistidas, não perda de registros.
UnB: **4 GET + 1 POST = 5**, 14 unidades persistidas.
Em ambas, a pesquisa depois da coleta efetuou **0 HTTP**.
Os FAIL live antigos permanecem como histórico das versões
anteriores; não foram reclassificados retroativamente.

A [execução 42181899282](https://github.com/franklinbaldo/ies/actions/runs/42181899282)
verificou a **saída humana real das sugestões** em UnB, com índice
SQLite *sintético*, montado após login autenticado (PASS,
**5 requisições**). Após intervalo de 30 segundos, o comando
`processo enviar 23106.001868/2026-91 --para Unida`
foi executado **sem `--confirmar`** e terminou com código 1,
`destino_nao_oferecido_pelo_sei`, mostrando no stderr
`Unidade2 - Unidade de treino`, «sugestões guardadas»
e «não confirmadas pelo SEI»; stdout vazio.
Resultado: **PASS**, **escritas de domínio 0**, **resíduo nenhum**.
Os logs publicaram apenas condições booleanas saneadas, não
cookie, URL assinada, senha ou HTML privado.
As requisições feitas **pela prévia** não foram contadas pelo
roteiro e devem constar como **não observadas**, não zero.
Na execução anterior [42181693551](https://github.com/franklinbaldo/ies/actions/runs/42181693551),
o preparo do snapshot sintético falhou após o login porque o
esquema SQLite já havia sido migrado. A preparação foi corrigida;
a execução anterior nem chegou à prévia.

O [CI anterior 42181906085](https://github.com/franklinbaldo/ies/actions/runs/42181906085)
passou **5/5 jobs** no head `9f9eab1`: prek, release, replay da
lupa GO/UnB, replay de processo criar e replay de retornos.
A atualização deste relatório e a remoção do workflow temporário
exigem **novo CI** no head final; não atribuir ao novo SHA
o CI do anterior.

**Situação:** coleta integral autenticada, persistência, pesquisa
posterior com zero HTTP e sugestões na saída humana agora têm
evidência real em treinamento. Não houve escrita de domínio no
SEI nem resíduo. Encerramento depende da revisão do head final
e da decisão da rotina integradora.
