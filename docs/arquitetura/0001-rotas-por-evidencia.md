---
type: ArchitectureDecision
title: Seleção de acesso ao SEI por evidência
description: Independência entre contrato de domínio e mecanismo de transporte
status: proposed
decision: domain-first evidence-gated adapters
---

# 0001 — Seleção de acesso ao SEI por evidência

## Problema

O IES Rust não é uma tradução do Todos Python. O Todos e suas 495
cassettes fornecem oráculos de comportamento e descobertas empíricas.
Eles não determinam como o IES deve ser implementado.

A navegação por HTML pode ser necessária para uma instalação, mas
não é necessariamente a forma mais simples de realizar cada operação.
Há interfaces de integração próprias do SEI que devem ser consideradas.

## Alternativas independentes

1. **REST WSSEI**, quando o módulo oficial estiver instalado, habilitado
   e o usuário/sistema tiver acesso autorizado. A API documenta
   `GET /processo/listar` (parâmetros `limit`, `start`, `tipo`,
   cabeçalhos `token` e `unidade`), `GET /processo/{protocolo}`,
   `GET /usuario/unidades`, além de operações de documentos e escritas.
   Isso é evidência de uma **interface disponível no código do módulo**,
   não de que exista em qualquer instalação ou preserve exatamente
   a semântica da caixa HTML.
2. **WebServices oficiais do SEI (SOAP/WSDL)**, quando o administrador
   houver cadastrado o sistema, serviços, escopos e forma de autenticação.
   Não presumir permissão a partir do acesso à interface Web.
3. **Web observada**: páginas, links assinados, formulários e transporte
   HTTP, usados apenas nos fluxos cujos gestos e efeitos foram demonstrados
   por instalação. Útil sem instalação de módulo; nunca montar
   `infra_hash`, explorar rotas não oferecidas nem inferir POSTs.
4. **Índice local derivado de observações autorizadas**: acelera
   pesquisas já observadas, mas não é uma fonte independente de verdade.
   Não deve autorizar ações nem substituir releituras de estado mutável.

## Decisão provisória

**O contrato público é de domínio.** Para cada operação, o IES pode
ter implementações pequenas e explícitas, mas nenhuma é escolhida
sem evidência de disponibilidade, escopo e equivalência semântica.

A escolha não é uniforme para todo o aplicativo. O mesmo contrato
pode ter um caminho REST comprovado numa instalação e precisar de
Web comprovada em outra. SOAP é alternativa condicionada a cadastro
institucional, não requisito de instalação do IES.

Não reintroduzir despachante genérico `Action`, `agir` nem
`PageModel`. Preferir funções por operação a um mecanismo que
descubra e execute ações arbitrárias.

Para **leituras**, alternativas são comparadas por completude,
identidades, paginação, ordem, campos, escopo de unidade, políticas
de permissão e custo. Uma resposta REST de processos não é, sem
teste, a mesma coisa que as duas listas da caixa Web.

Para **escritas**, exigem-se intenção explícita, prévia, confirmação,
identidade do alvo, observação de efeito e reconciliação após
resultado ambíguo. Nunca alternar automaticamente de backend
depois de erro/timeout de escrita.

A seleção automática só será admitida quando a disponibilidade
e a equivalência forem demonstradas; caso contrário, informar a
ausência de caminho aprovado ou solicitar configuração explícita.
Não sondar endpoints restritos ou usar credenciais de produção
sem autorização para experimentos.

## Invariantes da arquitetura

- Modelo de domínio e erros independentes do protocolo.
- Operação sem backend aprovado falha fechado, sem resposta simulada.
- Identificadores visíveis e internos não são intercambiáveis.
- Não publicar cookies, tokens, URLs assinadas ou dados de processos.
- A matriz OKF e as cassettes registram evidência, não *truth fields*.
- Medições com mesma instalação, conta, unidade, amostra e estado
  inicial; não comparar fluxos com resultados semanticamente distintos.
- Benchmarks devem contabilizar requests, latência, bytes, memória,
  custo de desenvolvimento e dependência administrativa.

## Contratos oficiais localizados

A [referência REST OpenAPI](../interfaces/rest-wssei.md) e a
[referência SOAP WSDL/XSD](../interfaces/soap-sei.md) são fontes
explicitamente distintas. A primeira é estática e versionada por commit;
a segunda depende do servidor que a disponibiliza. Nenhuma das duas
atesta equivalência com a caixa Web sem experimentação.

## Próximo experimento

Executar o [comparativo de interfaces](../experimentos/comparar-acessos-sei.md),
começando por `unidade.listar_processos`, `processo.obter` e
`documento.obter`. Primeiro comparar contratos e formatos offline;
testes live somente em instalações autorizadas e com controle de
sessão e credenciais.

## Fontes externas

- [Módulo REST oficial do PEN](https://www.gov.br/gestao/pt-br/assuntos/gestaoeinovacao/informacoes-sistemas-e-servicos-de-gestao/processo-eletronico-nacional/modulos/aplicativo-do-sei).
- [WSSEI e Swagger, projeto do desenvolvedor](https://github.com/pengovbr/mod-wssei/blob/c6bfce09bd181a5f245e9dbb596d5916dba69727/swagger/wssei.json).
- [Cadastro de sistemas e WebServices no SEI](https://manuais.processoeletronico.gov.br/pt-br/latest/SEIADM/Administracao_sei/Man_Adm_SEI_04-Administra%C3%A7%C3%A3o_SEI-11_Sistemas.html).
- [Como o SEI Web funciona](../como-o-sei-funciona/README.md).
