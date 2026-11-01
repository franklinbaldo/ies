---
type: ImplementationDecision
title: Leitura parcial da caixa da unidade por sessão Web existente
operation: unidade.listar_processos
implementation: partial
status: adopted-for-scope
scope: GET HTTPS autorizado da primeira tela do Controle de Processos com cobertura total verificada
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
Quando o total declarado exigir outras páginas, falhar explicitamente
em vez de responder como se todos os processos tivessem sido obtidos.

A opção local `--saida ARQUIVO` usa somente o resultado da mesma leitura
já aprovada, depois da verificação de cobertura, sem GET/POST adicional.
A exportação não deve sobrescrever arquivo existente nem publicar dados
dos processos no comprovante; testes cobrem confidencialidade no Unix
e falha segura de gravação.

## Evidências verificáveis

- [Contrato da operação](../../contracts/cli/unidade-listar-processos.md)
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
não implementa POST de paginação e não garante snapshot transacional.
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
e **nenhum POST**. A leitura Web ativa de múltiplas páginas permanece
bloqueada até prova por instalação dos controles, payload, possíveis
efeitos sobre a sessão e semântica das duas visões.

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
visualização, critérios, total e NUPs únicos; falta de página é erro,
não sucesso parcial. Arquivos HTML privados capturados podem complementar
o GET pela opção já existente, sem alegar proveniência criptográfica.

O orçamento de rede permanece **um GET e zero POST**. Não houve validação
autenticada em UnB/UFRJ/Goiás/PEN. A estratégia não autoriza automaticamente
mudar a visualização, navegar por links assinados ou enviar POST.
Implantar paginação ativa só após observar payload integral, efeito
administrativo e preservação da sessão em laboratório autorizado.
