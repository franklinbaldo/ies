---
type: ImplementationDecision
title: Leitura do prazo a partir do link assinado da barra e da tela observada
operation: processo.prazo
implementation: partial
status: adopted-for-scope
scope: ies processo prazo NUP leitura e prévia Web sem escrita; confirmação e exclusão fechadas
selected_strategy: web-link-da-barra-formulario-estrito
alternatives: web-link-da-barra-formulario-estrito,ler-pela-lista-da-unidade,rest-wssei
evidence_level: cassettes-publicas-unb-go-ufrj-e-testes-sinteticos
live_validation: not-run
---

# Controle de prazo (Refs #272)

A barra do nó `PROCESSO` publica `controle_prazo_definir`;
a tela autentica o processo por seu `hdnIdProcedimentos` e pela ação
do formulário. Foi escolhida a **leitura Web estrita** antes de qualquer
POST. O leitor recusa formas desconhecidas, não faz `retry`, não
normaliza URLs assinadas e usa `html_atributos`.

A alternativa da lista da unidade é indispensável para **exclusão**:
a barra não a oferece; a lista `controle_prazo_listar` usa
`acaoExcluir` e `controle_prazo_excluir`. Portanto não seria seguro
fingir que a exclusão se faz pelo formulário da barra.

Os testes sintéticos derivam dos casos de
`franklinbaldo/todos@323eb01:tests/test_ciclo_prazo_010.py`
(somente cenários) e verificam ano bissexto, exclusividade, leitura
dos estados, formulário alterado, id divergente e limpeza do campo
incompatível ao preparar a prévia.

Não converter `--confirmar` em um POST até implementar o protocolo de
reconciliação: prévia viva → envio uma vez → leitura nova, com saída 3
se não for possível demonstrar o efeito. A validação ao vivo só pode
ser feita em treinamento, e deve registrar o resíduo criado ou
alterado. A fatia atual não faz escrita.
