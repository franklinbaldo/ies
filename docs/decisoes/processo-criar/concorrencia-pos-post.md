---
type: ImplementationDecision
title: Sem atribuição causal a candidato único após POST criador sem resposta
description: A identidade e as propriedades de um processo novo não identificam o agente causador numa conta SEI compartilhada
operation: processo.criar
implementation: partial
status: adopted-for-scope
scope: reconciliacao-sem-resposta-e-concorrencia
selected_strategy: coincidencia-observada-e-efeito-incerto
alternatives: sucesso-por-candidato-unico,ignorar-candidatos,coincidencia-observada-e-efeito-incerto
evidence_level: interface-gerados-observada-e-regressao-sintetica
live_validation: not-run
---

# Decisão — coincidência não equivale a autoria (#245)

A caixa Gerados é compartilhada entre sessões/agentes da mesma unidade.
Após um POST sem resposta, um novo ID/NUP e propriedades exatamente
iguais ao pedido identificam um **possível resíduo**, mas não vinculam
esse processo ao POST investigado. Dois eventos causalmente distintos
(POST próprio aplicado e criação concorrente idêntica com POST próprio
não aplicado) produzem a mesma sequência de respostas observáveis.
Não existe predicado sobre essas fotografias que distinga os casos.

**Escolha:** `Desfecho::CoincidenciaUnica` significa apenas igualdade
observada. A camada de escrita retorna **saída 1**, diagnóstico tipado
`candidato_unico_sem_prova_causal`, NUP candidato e causa do POST.
O JSON indica `possivel_residuo`, sem `confirmado: true` e sem retry.
Quando há dois processos e apenas um coincide, é necessário verificar
ambos antes de classificar, mas a conclusão também é escrita não confirmada.
As provas parciais de candidatos seguem separadas.

Alternativas rejeitadas: declarar sucesso por um único candidato
produziria falso positivo com agente concorrente; descartar os NUPs
observados perderia evidência útil; repetir POST criaria duplicata.
Um `Location` vinculado ao POST, seguido de leitura independente,
permanece o caminho de sucesso. Recusa expressa da Infra permanece
prova de não aplicação; uma caixa Gerados vazia nunca é.

**Orçamento:** sem requisição nova: 3 GET prévios + 1 POST tentado +
1 GET Gerados + 3 GET por candidato; 8 com um candidato e 11 com
dois. Zero retries. A mudança foi motivada por observação já
documentada da estrutura Gerados e fixture sintética de concorrência,
não por ensaio ao vivo; a validação de escrita em segunda instalação
de treinamento continua pendente.

[Contrato de reconciliação](../../../contracts/cli/processo-criar/reconciliacao.md)
· [decisão geral](reconciliacao.md)
