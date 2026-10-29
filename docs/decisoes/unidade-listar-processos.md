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

## Evidências verificáveis

- [Contrato da operação](../../contracts/cli/unidade-listar-processos.md)
- [Matriz de evidências](../../contracts/evidencias/unidade-listar_processos.md)
- [Testes de autenticação e replay](../../tests/replay_unidade.rs)
- [Comparação experimental](../experimentos/comparar-acessos-sei.md)

## Limitações e revisão

O recorte não valida sessão live, não estabelece login persistente,
não implementa POST de paginação e não garante snapshot transacional.
Revisar a estratégia quando houver evidência autorizada de paginação
Web segura ou comprovação funcional de REST/SOAP por instalação.
A mudança de categoria de processos entre páginas impede inferir
estabilidade durante uma consulta.
