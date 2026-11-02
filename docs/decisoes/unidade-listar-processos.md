---
type: ImplementationDecision
title: Leitura da caixa da unidade por sessão Web existente
operation: unidade.listar_processos
implementation: partial
status: adopted-for-scope
scope: GET HTTPS autorizado do Controle de Processos com Recebidos e Gerados como listas independentes, total e faixa observados; um POST opcional por ies unidade recebidos|gerados --pagina N
selected_strategy: web-session-readonly
alternatives: web-session-readonly,rest-wssei,soap-official
evidence_level: replays-offline-e-contratos-documentados
live_validation: nao-realizada-neste-recorte
---

# Leitura da caixa da unidade

## Alternativas consideradas

- **web-session-readonly:** GET HTTPS da tela `procedimento_controlar`
  com sessão já existente e origem autorizada.
- **rest-wssei:** cliente da interface REST oficial, após comprovar
  disponibilidade e semântica específica nesta instalação.
- **soap-official:** serviço SOAP documentado, após autorização
  e comprovação de equivalência funcional.

## Comparação

O Web permite reaproveitar o parser e as verificações de páginas
já reproduzidas offline, mas a URL assinada expira e as páginas
posteriores não são obtidas por GET. REST e SOAP não devem ser
considerados funcionalmente equivalentes ao Controle de Processos
sem teste autorizado por instalação e identidade.

## Decisão

Adotar somente o GET Web autorizado como primeiro acesso real do
comando de domínio. Arquivos de URL, cookie e origem são privados
e referenciados por variáveis de ambiente contendo apenas caminhos.
Rejeitar origem diferente antes de ler cookies, não seguir redirects,
não efetuar POST ou login e não realizar troca automática de backend.
Quando o total declarado exigir outras páginas, devolver a página
observada com `total_declarado` e `faixa_observada`, sem afirmar lista
integral (decisão do proprietário, 02/11/2026; seção final).

A opção local `--saida ARQUIVO` usa somente o resultado da mesma leitura
já aprovada, depois da verificação de total e faixa, sem GET/POST adicional.
A exportação não deve sobrescrever arquivo existente nem publicar dados
dos processos no comprovante; testes cobrem confidencialidade no Unix
e falha segura de gravação.

## Evidências verificáveis

- [Contrato da operação](../../contracts/cli/unidade-listar-processos.md)
- [Páginas da caixa](../../contracts/cli/unidade-listar-processos-paginas.md)
- [Listas Recebidos e Gerados](../../contracts/cli/unidade-recebidos-gerados.md)
- [Testes da superfície das listas](../../tests/cli_unidade_listas.rs)
- [Matriz de evidências](../../contracts/evidencias/unidade-listar_processos.md)
- [Testes de autenticação e replay](../../tests/replay_unidade.rs)
- [Comparação experimental](../experimentos/comparar-acessos-sei.md)

A verificação do GET rejeita nomes de parâmetros PHP não canônicos e
separadores literais `;` para impedir ambiguidade de ação/hash após o
servidor decodificar a query. O teste de segurança permanece offline,
sem assumir gramática universal de URLs ainda não verificadas.

## Limitações e revisão

O replay pareado da cassette UnB `e02dbcfde8c4` demonstrou dois rótulos
`lnkInfraUnidade` nas barras móvel e desktop. Aceitar somente esse par
nas barras identificadas, com texto e href iguais; manter recusa para
unidades/destinos distintos, terceiro link e duplicações fora das barras.
O parser tl mantém a barra desktop dentro da móvel nessa resposta;
conferir as duas ocorrências distintas, sem assumir árvores disjuntas.
Evidência e medições: [ensaio pareado](../performance/paired-unit.md).

O recorte não valida sessão live, não estabelece login persistente,
não percorre todas as páginas e não garante snapshot transacional.
Revisar a estratégia quando houver evidência autorizada de paginação
Web segura ou comprovação funcional de REST/SOAP por instalação.
A mudança de categoria de processos entre páginas impede inferir
estabilidade durante uma consulta.


## Paginação da visão resumida — recusa de falso sucesso (Refs #57)

A referência comportamental do Todos distingue a visão resumida de
Recebidos/Gerados, sem paginação comprovada, da visão detalhada,
que publica `tblProcessosDetalhado` e utiliza
`hdnDetalhadoPaginaAtual` para avançar no formulário. A observação
reprodutível não autoriza extrapolar o campo de outras listas
(`hdnInfraPaginaAtual`) nem inventar um POST da caixa resumida.

Como recorte de integridade, o adaptador recusa controles explícitos
de paginação presentes em uma página `R` mesmo que a legenda
coincida com o número de linhas. A decisão evita afirmar cobertura
com base na primeira página; preserva o orçamento atual de **um GET**
e **nenhum POST**. Esses controles `lnkInfra*` não são os paginadores por
tabela da visão `R` (`lnk{T}*`, `sel{T}Paginacao*`), tratados na seção final.

## Leitura da tela detalhada no GET existente (Refs #57)

A matriz de derivação mantém `ies unidade listar-processos` como um
único comando. Quando a própria resposta de seu GET mostra
`tblProcessosDetalhado` com modo `D`, um leitor dedicado interpreta
a coluna `Processo`, o identificador técnico da linha e o link para
`procedimento_trabalhar`/`procedimento_visualizar`; não faz POST nem
inventaria categorias recebidos/gerados a partir desse layout distinto.
A projeção detalhada é explicitamente identificada na saída, sem
modificar o golden da visão resumida.

As faixas devem começar em 1, ser contíguas, preservar unidade,
visualização, critérios, total e NUPs únicos; página parcial publica
a faixa observada (seção final), sem afirmar lista integral. Arquivos HTML privados capturados podem complementar
o GET pela opção já existente, sem alegar proveniência criptográfica.

O orçamento de rede permanece **um GET e zero POST**. Não houve validação
autenticada em UnB/UFRJ/Goiás/PEN. A estratégia não autoriza automaticamente
mudar a visualização, navegar por links assinados ou enviar POST.
Implantar paginação ativa só após observar payload integral, efeito
administrativo e preservação da sessão em laboratório autorizado.

## Página observada e `--pagina N` (Refs #57, 02/11/2026)

Alternativas avaliadas para a caixa paginada: (a) recusar sempre
(`paginas_do_controle_nao_obtidas`, comportamento anterior); (b) percorrer
todas as páginas por POST, com teto e restauração; (c) devolver a página
observada com total e faixa, e pedir uma página específica por opt-in.
O proprietário escolheu (c), como [invariante geral](../../contracts/cli.md)
de leituras paginadas. (a) impedia qualquer leitura em PEN1/PEN3/PEN4 e
Goiás; (b) multiplicava requisições com efeito de sessão (Goiás exigiria
813 POSTs) e precisava de restauração, que também altera estado.

`--pagina N` não é flag substituta no sentido da
[matriz](../matriz-derivacao-cli.md): o comando existente não ganha outra
capacidade, só a autorização explícita para um POST com efeito
`PaginaOuFiltroLembrado`. Exige `--categoria` porque Recebidos e Gerados
têm paginadores independentes (`hdn{T}PaginaAtual`, `lnk{T}*`,
`sel{T}Paginacao*`) e cada ação do navegador muda só um deles.

Evidência estrutural (capturas privadas de 02/11/2026, só estrutura):
PEN1, PEN3 e PEN4 (5.0.4) e Goiás (5.0.3) chamam
`infraAcaoPaginar('+', 0, 'T', null)` em `lnk{T}ProximaPagina*` e
`infraAcaoPaginar('=', i, 'T', null)` em `lnk{T}UltimaPagina*` e no
`onchange` de `sel{T}Paginacao*`; o índice começa em 0 (Goiás: último
índice 798 para 7990 registros em páginas de 10). O formulário
`frmProcedimentoControlar` não tem botão de envio. O `InfraUtil.js` que
define `infraAcaoPaginar` **não** foi capturado: que a função só grava
`hdn{T}PaginaAtual` e chama `submit()` vem do oráculo Todos (listas comuns),
não da caixa. Ensaio offline do plano e da derivação nas capturas: PASS
(PEN1, PEN3, PEN4, Goiás; UnB e UFRJ sem paginação). Ensaio live: **NOT_RUN**.

Sem restauração da página 1: a faixa publicada mostra qual página a tela
traz, e a restauração seria outro POST com o mesmo efeito. Revisar se o
ensaio live mostrar critério ou unidade alterados após o POST, resposta
302, ou campo de página diferente do previsto.

## Listas independentes e subcomandos (decisão do proprietário, 02/11/2026)

Recebidos e Gerados são duas listas independentes do SEI: cada uma tem
paginador, total, faixa e página lembrada próprios; só compartilham a
resposta HTTP. Alternativas: (a) manter o objeto `{recebidos, gerados}`
e `--categoria T --pagina N` (PR #146); (b) forma única de lista e
subcomandos `ies unidade recebidos` e `ies unidade gerados`. O
proprietário escolheu (b), **por decisão explícita que se sobrepõe** à
regra da [matriz](../matriz-derivacao-cli.md) de não criar verbos: cada
lista vira um comando com `--pagina N`, e `listar-processos` devolve o
array `[recebidos, gerados]` com 1 GET, sem `--categoria` nem `--pagina`
(contrato pré-1.0, sem alias). A visão D devolve uma lista
`categoria: "detalhada"`. A operação e a estratégia não mudam: mesmo
GET, mesmo POST da #146, mesmas recusas; `--unidade` é conferido pela
mesma leitura. A exportação `--saida` e a importação `--caixa-local`
passam ao array; o formato anterior é recusado. Ensaio live da nova
sintaxe: **NOT_RUN** (o ensaio de 02/11/2026 usou `--categoria`).
