---
type: ImplementationDecision
title: Prévia do relacionamento pelo formulário e complementar servidos
operation: processo.relacionar
implementation: partial
status: adopted-for-scope
scope: ies processo relacionar NUP --com NUP com prévia apenas
selected_strategy: web-barra-formulario-complementar-e-linhas
alternatives: web-barra-formulario-complementar-e-linhas,rest,html-solto
evidence_level: tela-todos-pen1-e-testes-sinteticos
live_validation: not-run
---

# Relacionar processos — recorte #258

O Todos @323eb01 é fonte dos casos, não da arquitetura. A tela de
relacionamento PEN1 demonstra o complementar protocolo_RI1023, a tabela
Lista de Processos Relacionados e a remoção vinculada à linha. A cassette
PEN1 registra a operação sem que seu corpo HTTP tenha sido reproduzido
integralmente nesta execução.

Primeira fatia: ler a barra e a tabela via html_atributos, identificar destino
pela consulta AJAX de leitura com formulário_web, rejeitar autocorrelação e
repetições. Só uma linha da tabela é evidência de vínculo: não buscar
números livres pelo HTML.

Confirmação continua bloqueada antes da sessão: falta implementar e testar
POST único e releitura independente para adição e remoção, com saída 3 se
efeito incerto. Nenhum retry nem desligamento de gate é permitido. Esta
evidência PEN1 não presume paridade nas outras instalações. Live NOT_RUN.
